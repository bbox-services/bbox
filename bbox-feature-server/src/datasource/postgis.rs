use crate::datasource::{CollectionDatasource, CollectionInfo, ItemsResult};
use crate::endpoints::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_common::ogcapi::*;
use futures::TryStreamExt;
use log::warn;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Result, Row};

#[derive(Clone)]
pub struct PgDatasource {
    id: String,
    pool: PgPool,
}

#[derive(Clone, Debug)]
pub struct PgCollectionInfo {
    table_schema: String,
    table_name: String,
    geometry_column: String,
    /// Primary key column, None if multi column key.
    pk_column: Option<String>,
}

impl PgDatasource {
    pub async fn new_pool(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgDatasource {
            id: url.to_string(),
            pool,
        })
    }
}

#[async_trait]
impl CollectionDatasource for PgDatasource {
    async fn collections(&self) -> Result<Vec<FeatureCollection>> {
        let mut collections = Vec::new();
        let sql = r#"
            SELECT contents.*
            FROM geometry_columns contents
              JOIN spatial_ref_sys refsys ON refsys.srid = contents.srid
        "#;
        let mut rows = sqlx::query(&sql).fetch(&self.pool);
        while let Some(row) = rows.try_next().await? {
            let table_schema: String = row.try_get("f_table_schema")?;
            let table_name: String = row.try_get("f_table_name")?;
            let info = table_info(&self.pool, &table_schema, &table_name).await?;
            let id = &table_name.clone();
            let bbox = query_bbox(&self.pool, &info)
                .await
                .unwrap_or(vec![-180.0, -90.0, 180.0, 90.0]);
            let collection = CoreCollection {
                id: id.clone(),
                title: Some(id.clone()),
                description: None,
                extent: Some(CoreExtent {
                    spatial: Some(CoreExtentSpatial {
                        bbox: vec![bbox],
                        crs: None,
                    }),
                    temporal: None,
                }),
                item_type: None,
                crs: vec![],
                links: vec![ApiLink {
                    href: format!("/collections/{id}/items"),
                    rel: Some("items".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some(id.clone()),
                    hreflang: None,
                    length: None,
                }],
            };
            let fc = FeatureCollection {
                collection,
                info: CollectionInfo::PgCollectionInfo(info),
                ds_id: self.id.clone(),
            };
            collections.push(fc);
        }
        Ok(collections)
    }

    async fn items(&self, info: &CollectionInfo, filter: &FilterParams) -> Result<ItemsResult> {
        let CollectionInfo::PgCollectionInfo(info) = info else {
            panic!("Wrong CollectionInfo type");
        };
        let geometry_column = &info.geometry_column;
        let schema = &info.table_schema;
        let table = &info.table_name;
        let mut sql = if let Some(pk) = &info.pk_column {
            format!(
                r#"SELECT to_jsonb(t.*)-'{geometry_column}'-'{pk}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                      "{pk}"::varchar AS pk,
                      count(*) OVER () AS __total_cnt 
                   FROM "{schema}"."{table}" t"#,
            )
        } else {
            format!(
                r#"SELECT to_jsonb(t.*)-'{geometry_column}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                      NULL AS pk,
                      --row_number() OVER () ::varchar AS pk,
                      count(*) OVER () AS __total_cnt 
               FROM "{schema}"."{table}" t"#,
            )
        };
        let limit = filter.limit_or_default();
        if limit > 0 {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let number_matched = if let Some(row) = rows.first() {
            row.try_get::<i64, _>("__total_cnt")? as u64
        } else {
            0
        };
        let number_returned = rows.len() as u64;
        let items = rows
            .iter()
            .map(|row| row_to_feature(&row, &info))
            .collect::<Result<Vec<_>>>()?;
        let result = ItemsResult {
            features: items,
            number_matched,
            number_returned,
        };
        Ok(result)
    }

    async fn item(
        &self,
        info: &CollectionInfo,
        collection_id: &str,
        feature_id: &str,
    ) -> Result<Option<CoreFeature>> {
        let CollectionInfo::PgCollectionInfo(info) = info else {
            panic!("Wrong CollectionInfo type");
        };
        let Some(pk) = &info.pk_column else {
            warn!("Ignoring error getting item for {collection_id} without single primary key");
            return Ok(None)
        };
        let sql = format!(
            r#"SELECT to_jsonb(t.*)-'{geometry_column}'-'{pk}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                "{pk}"::varchar AS pk
               FROM "{schema}"."{table}" t
               WHERE {pk}::varchar = '{feature_id}'
            "#,
            geometry_column = &info.geometry_column,
            schema = &info.table_schema,
            table = &info.table_name
        );
        if let Some(row) = sqlx::query(&sql)
            // .bind(feature_id)
            .fetch_optional(&self.pool)
            .await?
        {
            let mut item = row_to_feature(&row, &info)?;
            item.links = vec![
                ApiLink {
                    href: format!("/collections/{collection_id}/items/{feature_id}"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: format!("/collections/{collection_id}"),
                    rel: Some("collection".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("the collection document".to_string()),
                    hreflang: None,
                    length: None,
                },
            ];
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }
}

async fn query_bbox(pool: &PgPool, info: &PgCollectionInfo) -> Result<Vec<f64>> {
    // TODO: Transform to WGS84, if necessary
    let sql = &format!(
        r#"
        WITH extent AS (
          SELECT ST_Extent("{geometry_column}") AS bbox
          FROM "{schema}"."{table}"
        )
        SELECT ST_XMin(bbox), ST_YMin(bbox), ST_XMax(bbox), ST_YMax(bbox)
        FROM extent
    "#,
        geometry_column = &info.geometry_column,
        schema = &info.table_schema,
        table = &info.table_name
    );
    let row = sqlx::query(sql).fetch_one(pool).await?;
    let extent: Vec<f64> = vec![
        row.try_get(0)?,
        row.try_get(1)?,
        row.try_get(2)?,
        row.try_get(3)?,
    ];
    Ok(extent)
}

async fn table_info(pool: &PgPool, schema: &str, table: &str) -> Result<PgCollectionInfo> {
    let sql = &format!(
        r#"
        WITH pkeys AS (
            SELECT a.attname
            FROM   pg_index i
            JOIN   pg_attribute a ON a.attrelid = i.indrelid
                                 AND a.attnum = ANY(i.indkey)
            WHERE  i.indrelid = '{schema}.{table}'::regclass
            AND    i.indisprimary
        )
        SELECT f_geometry_column,
          (SELECT COUNT(*) FROM pkeys) AS pksize,
          (SELECT attname FROM pkeys LIMIT 1) AS pk
        FROM geometry_columns
          JOIN spatial_ref_sys refsys ON refsys.srid = geometry_columns.srid
        WHERE f_table_schema = '{schema}' AND f_table_name = '{table}'
        "#
    );

    let row = sqlx::query(sql)
        // .bind(schema)
        // .bind(table)
        .fetch_one(pool)
        .await?;
    let geometry_column: String = row.try_get("f_geometry_column")?;
    let pksize: i64 = row.try_get("pksize")?;
    let pk_column: Option<String> = if pksize == 1 {
        row.try_get("pk")?
    } else {
        None
    };
    Ok(PgCollectionInfo {
        table_schema: schema.to_string(),
        table_name: table.to_string(),
        geometry_column,
        pk_column,
    })
}

fn row_to_feature(row: &PgRow, _table_info: &PgCollectionInfo) -> Result<CoreFeature> {
    let properties: serde_json::Value = row.try_get("properties")?;
    // properties[col.name()] = match col.type_info().name() {
    //     "VARCHAR"|"TEXT" => json!(row.try_get::<Option<&str>, _>(col.ordinal())?),
    //     "INT4" => json!(row.try_get::<Option<i32>, _>(col.ordinal())?),
    //     "INT8" => json!(row.try_get::<Option<i64>, _>(col.ordinal())?),
    //     "FLOAT4" => json!(row.try_get::<Option<f32>, _>(col.ordinal())?),
    //     "FLOAT8" => json!(row.try_get::<Option<f64>, _>(col.ordinal())?),
    //     ty => json!(format!("<{ty}>")),
    // }
    let geometry: serde_json::Value = row.try_get("geometry")?;
    // ERROR:  lwgeom_to_geojson: 'CurvePolygon' geometry type not supported
    let id: Option<String> = row.try_get("pk")?;

    let item = CoreFeature {
        type_: "Feature".to_string(),
        id,
        geometry,
        properties: Some(properties),
        links: vec![],
    };

    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    // docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb

    #[tokio::test]
    async fn pg_content() {
        let pool = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let collections = pool.collections().await.unwrap();
        assert!(collections.len() >= 3);
        assert!(collections
            .iter()
            .find(|col| col.collection.id == "ne_10m_rivers_lake_centerlines")
            .is_some());
    }

    #[tokio::test]
    async fn pg_features() {
        let filter = FilterParams::default();
        let pool = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let info = PgCollectionInfo {
            table_schema: "ne".to_string(),
            table_name: "ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
        };
        let items = pool
            .items(&CollectionInfo::PgCollectionInfo(info), &filter)
            .await
            .unwrap();
        assert_eq!(items.features.len(), filter.limit_or_default() as usize);
    }
}

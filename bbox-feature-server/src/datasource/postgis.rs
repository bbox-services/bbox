use crate::config::PostgisCollectionCfg;
use crate::datasource::{
    AutoscanCollectionDatasource, CollectionDatasource, CollectionSource, CollectionSourceCfg,
    ConfiguredCollectionCfg, ItemsResult,
};
use crate::error::{Error, Result};
use crate::filter_params::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::ogcapi::*;
use bbox_core::pg_ds::PgDatasource;
use futures::TryStreamExt;
use log::{debug, error, info, warn};
use sqlx::{postgres::PgRow, Row};

pub type Datasource = PgDatasource;

#[async_trait]
impl CollectionDatasource for PgDatasource {
    async fn setup_collection(
        &mut self,
        cfg: &ConfiguredCollectionCfg,
        _extent: Option<CoreExtent>,
    ) -> Result<FeatureCollection> {
        info!("Setup Postgis Collection `{}`", &cfg.name);
        let CollectionSourceCfg::Postgis(ref srccfg) = cfg.source else {
            panic!();
        };

        let id = &cfg.name;
        if srccfg.table_name.is_none() && srccfg.sql.is_none() {
            return Err(Error::DatasourceSetupError(format!(
                "Datasource `{id}`: configuration `table_name` or `sql` missing"
            )));
        } else if srccfg.table_name.is_some() && srccfg.sql.is_some() {
            warn!("Datasource`{id}`: configuration `table_name` ignored, using `sql` instead");
        }
        let (pk_column, geometry_column, sql) = if let Some(table_name) = &srccfg.table_name {
            let public = "public".to_string();
            let table_schema = srccfg.table_schema.as_ref().unwrap_or(&public);
            let pk_column =
                srccfg
                    .fid_field
                    .clone()
                    .or(detect_pk(self, &table_schema, &table_name).await?);
            let geometry_column = detect_geometry(self, &table_schema, &table_name).await?;
            let sql = check_query(
                self,
                format!(r#"SELECT * FROM "{table_schema}"."{table_name}""#),
            )
            .await?;
            (pk_column, geometry_column, sql)
        } else {
            let pk_column = srccfg.fid_field.clone();
            // TODO: We should also allow user queries without geometry
            let geometry_column =
                srccfg
                    .geometry_field
                    .clone()
                    .ok_or(Error::DatasourceSetupError(format!(
                        "Datasource `{id}`: configuration `geometry_field` missing"
                    )))?;
            let sql = check_query(self, srccfg.sql.clone().expect("config checked")).await?;
            (pk_column, geometry_column, sql)
        };
        if pk_column.is_none() {
            warn!("Datasource `{id}`: `fid_field` missing - single item queries will be ignored");
        }
        let source = PgCollectionSource {
            ds: self.clone(),
            sql,
            geometry_column,
            pk_column,
        };

        let bbox = source
            .query_bbox()
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
            source: Box::new(source),
        };
        Ok(fc)
    }
}

#[async_trait]
impl AutoscanCollectionDatasource for PgDatasource {
    async fn collections(&mut self) -> Result<Vec<FeatureCollection>> {
        let mut collections = Vec::new();
        let sql = r#"
            SELECT contents.*
            FROM geometry_columns contents
              JOIN spatial_ref_sys refsys ON refsys.srid = contents.srid
        "#;
        let mut rows = sqlx::query(sql).fetch(&self.pool);
        while let Some(row) = rows.try_next().await? {
            let table_schema: String = row.try_get("f_table_schema")?;
            let table_name: String = row.try_get("f_table_name")?;
            let coll_cfg = ConfiguredCollectionCfg {
                source: CollectionSourceCfg::Postgis(PostgisCollectionCfg {
                    table_schema: Some(table_schema),
                    table_name: Some(table_name.clone()),
                    ..Default::default()
                }),
                name: table_name.clone(),
                title: Some(table_name),
                description: None,
            };
            let fc = self.setup_collection(&coll_cfg, None).await?;
            collections.push(fc);
        }
        Ok(collections)
    }
}

#[derive(Clone, Debug)]
pub struct PgCollectionSource {
    ds: PgDatasource,
    sql: String,
    geometry_column: String,
    /// Primary key column, None if multi column key.
    pk_column: Option<String>,
}

#[async_trait]
impl CollectionSource for PgCollectionSource {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult> {
        let geometry_column = &self.geometry_column;
        let mut sql = format!("WITH query AS ({sql})\n", sql = &self.sql);
        let select_sql = if let Some(pk) = &self.pk_column {
            format!(
                r#"SELECT to_jsonb(t.*)-'{geometry_column}'-'{pk}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                      "{pk}"::varchar AS pk,
                      count(*) OVER () AS __total_cnt 
                   FROM query t"#,
            )
        } else {
            format!(
                r#"SELECT to_jsonb(t.*)-'{geometry_column}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                      NULL AS pk,
                      --row_number() OVER () ::varchar AS pk,
                      count(*) OVER () AS __total_cnt 
               FROM query t"#,
            )
        };
        sql.push_str(&select_sql);
        match filter.bbox() {
            Ok(Some(bbox)) => {
                sql.push_str(&format!(
                    " WHERE {geometry_column} && ST_MakeEnvelope({xmin}, {ymin}, {xmax}, {ymax})",
                    xmin = bbox[0],
                    ymin = bbox[1],
                    xmax = bbox[2],
                    ymax = bbox[3],
                ));
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Ignoring invalid bbox: {e}");
            }
        }
        let limit = filter.limit_or_default();
        if limit > 0 {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        let rows = sqlx::query(&sql).fetch_all(&self.ds.pool).await?;
        let number_matched = if let Some(row) = rows.first() {
            row.try_get::<i64, _>("__total_cnt")? as u64
        } else {
            0
        };
        let number_returned = rows.len() as u64;
        let items = rows
            .iter()
            .map(|row| row_to_feature(row, self))
            .collect::<Result<Vec<_>>>()?;
        let result = ItemsResult {
            features: items,
            number_matched,
            number_returned,
        };
        Ok(result)
    }

    async fn item(&self, collection_id: &str, feature_id: &str) -> Result<Option<CoreFeature>> {
        let Some(pk) = &self.pk_column else {
            warn!("Ignoring error getting item for {collection_id} without single primary key");
            return Ok(None);
        };
        let sql = format!(
            r#"
            WITH query AS ({sql})
            SELECT to_jsonb(t.*)-'{geometry_column}'-'{pk}' AS properties, ST_AsGeoJSON({geometry_column})::jsonb AS geometry,
                "{pk}"::varchar AS pk
               FROM query t
               WHERE {pk}::varchar = '{feature_id}'"#,
            sql = &self.sql,
            geometry_column = &self.geometry_column,
        );
        if let Some(row) = sqlx::query(&sql)
            // .bind(feature_id)
            .fetch_optional(&self.ds.pool)
            .await?
        {
            let mut item = row_to_feature(&row, self)?;
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

fn row_to_feature(row: &PgRow, _table_info: &PgCollectionSource) -> Result<CoreFeature> {
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

impl PgCollectionSource {
    async fn query_bbox(&self) -> Result<Vec<f64>> {
        // TODO: Transform to WGS84, if necessary
        let sql = &format!(
            r#"
        WITH query AS ({sql})
        WITH extent AS (
          SELECT ST_Extent("{geometry_column}") AS bbox
          FROM query
        )
        SELECT ST_XMin(bbox), ST_YMin(bbox), ST_XMax(bbox), ST_YMax(bbox)
        FROM extent
    "#,
            sql = &self.sql,
            geometry_column = &self.geometry_column,
        );
        let row = sqlx::query(sql).fetch_one(&self.ds.pool).await?;
        let extent: Vec<f64> = vec![
            row.try_get(0)?,
            row.try_get(1)?,
            row.try_get(2)?,
            row.try_get(3)?,
        ];
        Ok(extent)
    }
}

async fn detect_pk(ds: &PgDatasource, schema: &str, table: &str) -> Result<Option<String>> {
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
        SELECT
          (SELECT COUNT(*) FROM pkeys) AS pksize,
          (SELECT attname FROM pkeys LIMIT 1) AS pk
        "#
    );
    let row = sqlx::query(sql).fetch_one(&ds.pool).await?;
    let pksize: i64 = row.try_get("pksize")?;
    let pk_column: Option<String> = if pksize == 1 {
        row.try_get("pk")?
    } else {
        None
    };
    Ok(pk_column)
}

async fn detect_geometry(ds: &PgDatasource, schema: &str, table: &str) -> Result<String> {
    let sql = &format!(
        r#"
        SELECT f_geometry_column
        FROM geometry_columns
          JOIN spatial_ref_sys refsys ON refsys.srid = geometry_columns.srid
        WHERE f_table_schema = '{schema}' AND f_table_name = '{table}'
        "#
    );
    let row = sqlx::query(sql)
        // .bind(schema)
        // .bind(table)
        // We take the first result only
        .fetch_one(&ds.pool)
        .await?;
    let geometry_column: String = row.try_get("f_geometry_column")?;
    Ok(geometry_column)
}

async fn check_query(ds: &PgDatasource, sql: String) -> Result<String> {
    debug!("Collection query: {sql}");
    // TODO: prepare only
    if let Err(e) = sqlx::query(&sql).fetch_one(&ds.pool).await {
        error!("Error in collection query `{sql}`: {e}");
        return Err(e.into());
    }
    Ok(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    // docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb

    #[tokio::test]
    #[ignore]
    async fn pg_content() {
        let mut pool =
            PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
                .await
                .unwrap();
        let collections = pool.collections().await.unwrap();
        assert!(collections.len() >= 3);
        assert!(collections
            .iter()
            .any(|col| col.collection.id == "ne_10m_rivers_lake_centerlines"));
    }

    #[tokio::test]
    #[ignore]
    async fn pg_features() {
        let filter = FilterParams::default();
        let ds = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let source = PgCollectionSource {
            ds,
            sql: "SELECT * FROM ne.ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), filter.limit_or_default() as usize);
    }

    #[tokio::test]
    #[ignore]
    async fn pg_bbox_filter() {
        let filter = FilterParams {
            limit: Some(50),
            offset: None,
            bbox: Some("633510.0904,5762740.4365,1220546.4677,6051366.6553".to_string()),
            // WGS84: 5.690918,45.890008,10.964355,47.665387
        };
        let ds = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let source = PgCollectionSource {
            ds,
            sql: "SELECT * FROM ne.ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 10);
    }
}

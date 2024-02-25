use crate::config::PostgisCollectionCfg;
use crate::datasource::{
    AutoscanCollectionDatasource, CollectionDatasource, CollectionSource, CollectionSourceCfg,
    ConfiguredCollectionCfg, ItemsResult,
};
use crate::error::{Error, Result};
use crate::filter_params::{FilterParams, TemporalType};
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_core::ogcapi::*;
use bbox_core::pg_ds::PgDatasource;
use futures::TryStreamExt;
use log::{debug, error, info, warn};
use sqlx::{postgres::PgRow, Postgres, QueryBuilder, Row};
use std::collections::HashMap;

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
        let temporal_column = srccfg.temporal_field.clone();
        let (pk_column, geometry_column, sql) = if let Some(table_name) = &srccfg.table_name {
            let public = "public".to_string();
            let table_schema = srccfg.table_schema.as_ref().unwrap_or(&public);
            let pk_column = srccfg
                .fid_field
                .clone()
                .or(detect_pk(self, table_schema, table_name).await?);
            let geometry_column = detect_geometry(self, table_schema, table_name).await?;
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
        let other_columns = if let Some(fields) = srccfg.queryable_fields.clone() {
            let mut hm = HashMap::new();
            for field in fields {
                hm.insert(field, 0);
            }
            hm
        } else {
            HashMap::new()
        };

        let source = PgCollectionSource {
            ds: self.clone(),
            sql,
            geometry_column,
            pk_column,
            temporal_column,
            other_columns,
        };

        let bbox = source
            .query_bbox()
            .await
            .unwrap_or(vec![-180.0, -90.0, 180.0, 90.0]);
        let collection = CoreCollection {
            id: id.clone(),
            title: Some(id.clone()),
            description: cfg.description.clone(),
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
    temporal_column: Option<String>,
    /// Queriable columns.
    other_columns: HashMap<String, u8>,
}

#[async_trait]
impl CollectionSource for PgCollectionSource {
    async fn items(&self, filter: &FilterParams) -> Result<ItemsResult> {
        let geometry_column = &self.geometry_column;
        let temporal_column = &self.temporal_column;
        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new(format!("WITH query AS ({sql})\n", sql = &self.sql));
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
        builder.push(&select_sql);
        let mut where_term = false;
        match filter.bbox() {
            Ok(Some(bbox)) => {
                builder.push(format!(" WHERE ( {geometry_column} && ST_MakeEnvelope("));
                let mut separated = builder.separated(",");
                separated.push_bind(bbox[0]);
                separated.push_bind(bbox[1]);
                separated.push_bind(bbox[2]);
                separated.push_bind(bbox[3]);
                builder.push(") ) ");
                where_term = true;
            }
            Ok(None) => {}
            Err(e) => {
                error!("Ignoring invalid bbox: {e}");
                return Err(Error::QueryParams);
            }
        }
        if let Some(temporal_column) = temporal_column {
            match filter.temporal() {
                Ok(Some(parts)) => {
                    if where_term {
                        builder.push(" AND ");
                    } else {
                        builder.push(" WHERE ");
                        where_term = true;
                    }
                    if parts.len() == 1 {
                        if let TemporalType::DateTime(dt) = parts[0] {
                            builder.push(format!(" {temporal_column} = '{}'", dt,));
                        }
                    } else {
                        match parts[0] {
                            TemporalType::Open => match parts[1] {
                                TemporalType::Open => {
                                    error!("Open to Open datetimes doesn't make sense");
                                    return Err(Error::QueryParams);
                                }
                                TemporalType::DateTime(dt) => {
                                    builder.push(format!(" {temporal_column} <= '{}'", dt,));
                                }
                            },
                            TemporalType::DateTime(dt1) => match parts[1] {
                                TemporalType::Open => {
                                    builder.push(format!(" {temporal_column} >= '{}'", dt1,));
                                }
                                TemporalType::DateTime(dt2) => {
                                    builder.push(format!(
                                        " {temporal_column} >= '{}' and {temporal_column} <= '{}'",
                                        dt1, dt2
                                    ));
                                }
                            },
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    error!("Ignoring invalid temporal field: {e}");
                    return Err(Error::QueryParams);
                }
            }
        }

        match filter.other_params() {
            Ok(others) => {
                if !others.is_empty() {
                    if where_term {
                        builder.push(" AND ");
                    } else {
                        builder.push(" WHERE ");
                    }
                }
                let mut separated = builder.separated(" AND ");
                for (key, val) in others {
                    // check if the passed in field matches queryables
                    // detect if value has wildcards
                    if self.other_columns.get(key).is_some() {
                        let val = if val.rfind('*').is_some() {
                            separated.push(format!("{key} like "));
                            val.replace('*', "%")
                        } else {
                            separated.push(format!("{key}="));
                            val.to_string()
                        };
                        separated.push_bind_unseparated(val);
                        /*
                        match key_type.as_str() {
                            "String" => {
                                separated.push_bind_unseparated(val);
                            },
                            "Numeric" => {
                                let num = match val.parse::<i64>() {
                                    Ok(n) => n,
                                    Err(_) => return Err(Error::QueryParams)
                                };
                                separated.push_bind_unseparated(num);
                            },
                            &_ => {
                                return Err(Error::QueryParams)
                            },
                        }
                            */
                    } else {
                        error!("Invalid query param {key}");
                        return Err(Error::QueryParams);
                    }
                }
            }
            Err(e) => {
                error!("{e}");
                return Err(Error::QueryParams);
            }
        }
        let limit = filter.limit_or_default();
        if limit > 0 {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }
        debug!("SQL: {}", builder.sql());
        let query = builder.build();
        let rows = query.fetch_all(&self.ds.pool).await?;
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
        WITH query AS ({sql}),
        extent AS (
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
    use std::collections::HashMap;
    use test_log::test;

    // docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb

    #[test(tokio::test)]
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

    #[test(tokio::test)]
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
            temporal_column: None,
            other_columns: HashMap::new(),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), filter.limit_or_default() as usize);
    }

    #[test(tokio::test)]
    #[ignore]
    async fn pg_bbox_filter() {
        let filter = FilterParams {
            limit: Some(50),
            offset: None,
            bbox: Some("633510.0904,5762740.4365,1220546.4677,6051366.6553".to_string()),
            // WGS84: 5.690918,45.890008,10.964355,47.665387
            datetime: None,
            filters: HashMap::new(),
        };
        let ds = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let source = PgCollectionSource {
            ds,
            sql: "SELECT * FROM ne.ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
            temporal_column: None,
            other_columns: HashMap::new(),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 10);
    }

    #[test(tokio::test)]
    #[ignore]
    async fn pg_datetime_filter() {
        let ds = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let source = PgCollectionSource {
            ds,
            sql: "SELECT *, '2024-01-01 00:00:00Z'::timestamptz - (fid-1) * INTERVAL '1 day' AS ts FROM ne.ne_10m_rivers_lake_centerlines ORDER BY fid".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
            temporal_column: Some("ts".to_string()),
            other_columns: HashMap::new(),
        };

        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: Some("2023-10-14T00:00:00Z".to_string()),
            filters: HashMap::new(),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 1);

        // Combined with bbox
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: Some("633510.0904,5762740.4365,1220546.4677,6051366.6553".to_string()),
            datetime: Some("2023-10-14T00:00:00Z".to_string()),
            filters: HashMap::new(),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 1);

        // Outside of bbox
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: Some("633510.0904,5762740.4365,1220546.4677,6051366.6553".to_string()),
            datetime: Some("2024-01-01T00:00:00Z".to_string()),
            filters: HashMap::new(),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 0);
    }

    #[test(tokio::test)]
    #[ignore]
    async fn pg_field_filter() {
        let ds = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let source = PgCollectionSource {
            ds,
            sql: "SELECT *, '2024-01-01 00:00:00Z'::timestamptz - (fid-1) * INTERVAL '1 day' AS ts FROM ne.ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "wkb_geometry".to_string(),
            pk_column: Some("fid".to_string()),
            temporal_column: Some("ts".to_string()),
            other_columns: HashMap::from([("name".to_string(), 0)]),
        };

        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: None,
            filters: HashMap::from([("name".to_string(), "Rhein".to_string())]),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 2);

        // Existing filter column, but not queriable
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: None,
            filters: HashMap::from([("scalerank".to_string(), "4".to_string())]),
        };
        assert!(source.items(&filter).await.is_err());

        // Existing filter column, but not queriable
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: None,
            filters: HashMap::from([("foo".to_string(), "bar".to_string())]),
        };
        assert!(source.items(&filter).await.is_err());

        // Combined with bbox
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: Some("633510.0904,5762740.4365,1220546.4677,6051366.6553".to_string()),
            // WGS84: 5.690918,45.890008,10.964355,47.665387
            datetime: None,
            filters: HashMap::from([("name".to_string(), "Rhein".to_string())]),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 2);

        // outside bbox
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: Some("633510.0904,5762740.4365,633511,5762741".to_string()),
            datetime: None,
            filters: HashMap::from([("name".to_string(), "Rhein".to_string())]),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 0);

        // Combined with datetime
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: Some("2023-10-14T00:00:00Z".to_string()),
            filters: HashMap::from([("name".to_string(), "Rhein".to_string())]),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 1);

        // Other datetime
        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
            datetime: Some("2023-10-01T00:00:00Z".to_string()),
            filters: HashMap::from([("name".to_string(), "Rhein".to_string())]),
        };
        let items = source.items(&filter).await.unwrap();
        assert_eq!(items.features.len(), 0);
    }
}

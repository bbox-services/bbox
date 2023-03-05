use crate::datasource::{CollectionDatasource, CollectionInfo, ItemsResult};
use crate::endpoints::FilterParams;
use crate::inventory::FeatureCollection;
use async_trait::async_trait;
use bbox_common::ogcapi::*;
use futures::TryStreamExt;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Result, Row};

#[derive(Clone)]
pub struct PgDatasource {
    pool: PgPool,
}

#[derive(Clone, Debug)]
pub struct PgCollectionInfo {
    table_schema: String,
    table_name: String,
    geometry_column: String,
}

impl PgDatasource {
    pub async fn new_pool(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect(url)
            .await?;
        Ok(PgDatasource { pool })
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
            let geometry_column: String = row.try_get("f_geometry_column")?;
            let id = &table_name.clone();
            let info = PgCollectionInfo {
                table_schema,
                table_name,
                geometry_column,
            };
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
            };
            collections.push(fc);
        }
        Ok(collections)
    }

    async fn items(&self, info: &CollectionInfo, filter: &FilterParams) -> Result<ItemsResult> {
        let CollectionInfo::PgCollectionInfo(info) = info else {
            panic!("Wrong CollectionInfo type");
        };
        todo!()
    }

    async fn item(
        &self,
        info: &CollectionInfo,
        collection_id: &str,
        feature_id: &str,
    ) -> Result<Option<CoreFeature>> {
        todo!()
    }
}

async fn query_bbox(pool: &PgPool, info: &PgCollectionInfo) -> Result<Vec<f64>> {
    // TODO: Transform to WGS84, if necessary
    let sql = &format!(
        r#"
        WITH extent AS (
          SELECT ST_Extent("{}") AS bbox
          FROM "{}"."{}"
        )
        SELECT ST_XMin(bbox), ST_YMin(bbox), ST_XMax(bbox), ST_YMax(bbox)
        FROM extent
    "#,
        info.geometry_column, info.table_schema, info.table_name
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

    // #[tokio::test]
    async fn pg_features() {
        let filter = FilterParams::default();
        let pool = PgDatasource::new_pool("postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests")
            .await
            .unwrap();
        let info = PgCollectionInfo {
            table_schema: "public".to_string(),
            table_name: "ne_10m_rivers_lake_centerlines".to_string(),
            geometry_column: "geom".to_string(),
        };
        let items = pool
            .items(&CollectionInfo::PgCollectionInfo(info), &filter)
            .await
            .unwrap();
        assert_eq!(items.features.len(), filter.limit_or_default() as usize);
    }
}

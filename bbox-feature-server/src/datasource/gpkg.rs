use crate::datasource::ItemsResult;
use crate::endpoints::FilterParams;
use bbox_common::ogcapi::*;
use geozero::{geojson, wkb};
use serde_json::json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Result, Row, TypeInfo};

#[derive(Clone, Debug)]
pub struct SqliteConnections(SqlitePool);

impl SqliteConnections {
    pub async fn new_pool(gpkg: &str) -> Result<Self> {
        let conn_options = SqliteConnectOptions::new().filename(gpkg).read_only(true);
        let pool = SqlitePoolOptions::new()
            .min_connections(0)
            .max_connections(8)
            .connect_with(conn_options)
            .await?;
        Ok(SqliteConnections(pool))
    }
}

impl SqliteConnections {
    pub async fn collections(&self) -> Result<Vec<CoreCollection>> {
        let sql = r#"
        SELECT contents.*
        FROM gpkg_contents contents
          JOIN gpkg_spatial_ref_sys refsys ON refsys.srs_id = contents.srs_id
          --JOIN gpkg_geometry_columns geom_cols ON geom_cols.table_name = contents.table_name
        WHERE data_type='features'
    "#;
        let rows = sqlx::query(&sql).fetch_all(&self.0).await?;
        let collections = rows
            .iter()
            .map(|row| {
                let id: String = row.try_get("table_name")?;
                let title: String = row.try_get("identifier")?;

                let collection = CoreCollection {
                    id: id.clone(),
                    title: Some(title.clone()),
                    description: row.try_get("description")?,
                    extent: Some(CoreExtent {
                        spatial: Some(CoreExtentSpatial {
                            bbox: vec![vec![
                                row.try_get("min_x")?,
                                row.try_get("min_y")?,
                                row.try_get("max_x")?,
                                row.try_get("max_y")?,
                            ]],
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
                        title: Some(title),
                        hreflang: None,
                        length: None,
                    }],
                };
                Ok(collection)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(collections)
    }

    pub async fn items(&self, table: &str, filter: &FilterParams) -> Result<ItemsResult> {
        let table_info = table_info(&self.0, table).await?;

        let mut sql = format!("SELECT *, count(*) OVER() AS __total_cnt FROM {table}"); // TODO: Sanitize table name
        let limit = filter.limit_or_default();
        if limit > 0 {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }
        let rows = sqlx::query(&sql).fetch_all(&self.0).await?;
        let number_matched = if let Some(row) = rows.first() {
            row.try_get::<u32, _>("__total_cnt")? as u64
        } else {
            0
        };
        let number_returned = rows.len() as u64;
        let items = rows
            .iter()
            .map(|row| row_to_feature(&row, &table_info))
            .collect::<Result<Vec<_>>>()?;
        let result = ItemsResult {
            features: items,
            number_matched,
            number_returned,
        };
        Ok(result)
    }

    pub async fn item(&self, table: &str, feature_id: &str) -> Result<Option<CoreFeature>> {
        let table_info = table_info(&self.0, table).await?;

        let sql = format!(
            "SELECT * FROM {table} WHERE {} = ?", // TODO: Sanitize table name
            table_info.pk_column.as_ref().unwrap()
        );
        if let Some(row) = sqlx::query(&sql)
            .bind(feature_id)
            .fetch_optional(&self.0)
            .await?
        {
            let mut item = row_to_feature(&row, &table_info)?;
            item.links = vec![
                ApiLink {
                    href: format!("/collections/{table}/items/{feature_id}"),
                    rel: Some("self".to_string()),
                    type_: Some("application/geo+json".to_string()),
                    title: Some("this document".to_string()),
                    hreflang: None,
                    length: None,
                },
                ApiLink {
                    href: format!("/collections/{table}"),
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

struct TableInfo {
    geom_column: String,
    #[allow(dead_code)]
    geometry_type_name: String,
    /// Primary key column, None if multi column key.
    pk_column: Option<String>,
}

async fn table_info(pool: &SqlitePool, table: &str) -> Result<TableInfo> {
    // TODO: support multiple geometry columns
    let sql = r#"
        SELECT column_name, geometry_type_name,
          (SELECT COUNT(*) FROM pragma_table_info(?) ti WHERE ti.pk > 0) as pksize,
          (SELECT ti.name FROM pragma_table_info(?) ti WHERE ti.pk = 1) as pk
        FROM gpkg_geometry_columns
        WHERE table_name = ?
    "#;
    let row = sqlx::query(sql)
        .bind(table)
        .bind(table)
        .bind(table)
        .fetch_one(pool)
        .await?;
    let geom_column: String = row.try_get("column_name")?;
    let geometry_type_name: String = row.try_get("geometry_type_name")?;
    let pksize: u16 = row.try_get("pksize")?;
    let pk_column: Option<String> = if pksize == 1 {
        Some(row.try_get("pk")?)
    } else {
        None
    };
    Ok(TableInfo {
        geom_column,
        geometry_type_name,
        pk_column,
    })
}

fn row_to_feature(row: &SqliteRow, table_info: &TableInfo) -> Result<CoreFeature> {
    let mut id = None;
    let mut properties = json!({});
    for col in row.columns() {
        if col.name() == table_info.geom_column {
            // Skip geometry
        } else if col.name() == "__total_cnt" {
            // Skip count
        } else if col.name() == table_info.pk_column.as_ref().unwrap_or(&"".to_string()) {
            // Get id as String
            id = match col.type_info().name() {
                "TEXT" => Some(row.try_get::<String, _>(col.ordinal())?),
                "INTEGER" => Some(row.try_get::<i64, _>(col.ordinal())?.to_string()),
                _ => None,
            }
        } else {
            properties[col.name()] = match col.type_info().name() {
                "TEXT" => json!(row.try_get::<&str, _>(col.ordinal())?),
                "INTEGER" => json!(row.try_get::<i64, _>(col.ordinal())?),
                "REAL" => json!(row.try_get::<f64, _>(col.ordinal())?),
                "DATETIME" => json!(row.try_get::<&str, _>(col.ordinal())?),
                ty => json!(format!("<{ty}>")),
            }
        }
    }
    let wkb: wkb::Decode<geojson::GeoJsonString> = row.try_get(table_info.geom_column.as_str())?;
    let geom = wkb.geometry.unwrap();

    let item = CoreFeature {
        type_: "Feature".to_string(),
        id,
        geometry: serde_json::from_str(&geom.0).unwrap(),
        properties: Some(properties),
        links: vec![],
    };

    Ok(item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gpkg_content() {
        let pool = SqliteConnections::new_pool("../data/ne_extracts.gpkg")
            .await
            .unwrap();
        let collections = pool.collections().await.unwrap();
        assert_eq!(collections.len(), 3);
        assert_eq!(
            collections
                .iter()
                .map(|col| col.id.clone())
                .collect::<Vec<_>>(),
            vec![
                "ne_10m_rivers_lake_centerlines",
                "ne_10m_lakes",
                "ne_10m_populated_places"
            ]
        );
    }

    #[tokio::test]
    async fn gpkg_features() {
        let filter = FilterParams::default();
        let pool = SqliteConnections::new_pool("../data/ne_extracts.gpkg")
            .await
            .unwrap();
        let items = pool.items("ne_10m_lakes", &filter).await.unwrap();
        assert_eq!(items.features.len(), filter.limit_or_default() as usize);
    }
}

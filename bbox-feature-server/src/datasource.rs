use bbox_common::ogcapi::*;
use geozero::{geojson, wkb};
use log::debug;
use serde_json::json;
use sqlx::sqlite::SqliteConnection;
use sqlx::sqlite::SqliteRow;
use sqlx::{Column, Connection, Result, Row, TypeInfo};

pub async fn gpkg_collections(gpkg: &str) -> Result<Vec<CoreCollection>> {
    debug!("Reading gpkg_contents of {gpkg}");
    let mut conn = SqliteConnection::connect(&format!("sqlite://{gpkg}")).await?;
    let sql = r#"
        SELECT contents.*
        FROM gpkg_contents contents
          JOIN gpkg_spatial_ref_sys refsys ON refsys.srs_id = contents.srs_id
          --JOIN gpkg_geometry_columns geom_cols ON geom_cols.table_name = contents.table_name
        WHERE data_type='features'
    "#;
    let rows = sqlx::query(&sql).fetch_all(&mut conn).await?;
    let collections = rows
        .iter()
        .map(|row| {
            let id: String = row.try_get("table_name").unwrap();
            let title: String = row.try_get("identifier").unwrap();

            CoreCollection {
                id: id.clone(),
                title: Some(title.clone()),
                description: row.try_get("description").unwrap(),
                extent: Some(CoreExtent {
                    spatial: Some(CoreExtentSpatial {
                        bbox: vec![vec![
                            row.try_get("min_x").unwrap(),
                            row.try_get("min_y").unwrap(),
                            row.try_get("max_x").unwrap(),
                            row.try_get("max_y").unwrap(),
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
            }
        })
        .collect();
    Ok(collections)
}

pub async fn gpkg_items(gpkg: &str, table: &str) -> Result<Vec<CoreFeature>> {
    let mut conn = SqliteConnection::connect(&format!("sqlite://{gpkg}")).await?;
    let table_info = table_info(&mut conn, table).await?;

    let sql = format!("SELECT * FROM {table}"); // TODO: Sanitize table name
    let rows = sqlx::query(&sql).fetch_all(&mut conn).await?;
    let items = rows
        .iter()
        .map(|row| row_properties(&row, &table_info).unwrap().unwrap())
        .collect();
    Ok(items)
}

pub async fn gpkg_item(gpkg: &str, table: &str, feature_id: &str) -> Result<Option<CoreFeature>> {
    let mut conn = SqliteConnection::connect(&format!("sqlite://{gpkg}")).await?;
    let table_info = table_info(&mut conn, table).await?;

    let sql = format!(
        "SELECT * FROM {table} WHERE {} = ?", // TODO: Sanitize table name
        table_info.pk_column.as_ref().unwrap()
    );
    let row = sqlx::query(&sql)
        .bind(feature_id)
        .fetch_one(&mut conn)
        .await?;
    let item = row_properties(&row, &table_info)?.and_then(|mut item| {
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
        Some(item)
    });
    Ok(item)
}

struct TableInfo {
    geom_column: String,
    #[allow(dead_code)]
    geometry_type_name: String,
    /// Primary key column, None if multi column key.
    pk_column: Option<String>,
}

async fn table_info(conn: &mut SqliteConnection, table: &str) -> Result<TableInfo> {
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
        .fetch_one(conn)
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

fn row_properties(row: &SqliteRow, table_info: &TableInfo) -> Result<Option<CoreFeature>> {
    let columns = row.columns();
    let mut properties = json!({});
    let mut id = None;
    columns
        .iter()
        .filter(|col| col.name() != table_info.geom_column)
        .for_each(|col| {
            if col.name() == table_info.pk_column.as_ref().unwrap_or(&"".to_string()) {
                // Get id as String
                id = match col.type_info().name() {
                    "TEXT" => Some(
                        row.try_get::<&str, usize>(col.ordinal())
                            .unwrap()
                            .to_string(),
                    ),
                    "INTEGER" => Some(
                        row.try_get::<i64, usize>(col.ordinal())
                            .unwrap()
                            .to_string(),
                    ),
                    _ => None,
                }
            } else {
                properties[col.name()] = match col.type_info().name() {
                    "TEXT" => json!(row.try_get::<&str, usize>(col.ordinal()).unwrap()),
                    "INTEGER" => json!(row.try_get::<i64, usize>(col.ordinal()).unwrap()),
                    ty => json!(format!("<{ty}>")),
                }
            }
        });
    let wkb: wkb::Decode<geojson::GeoJsonString> = row.try_get(table_info.geom_column.as_str())?;
    let geom = wkb.geometry.unwrap();

    let item = CoreFeature {
        type_: "Feature".to_string(),
        id,
        geometry: serde_json::from_str(&geom.0).unwrap(),
        properties: Some(properties),
        links: vec![],
    };

    Ok(Some(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gpkg_content() {
        let collections = gpkg_collections("../data/ne_extracts.gpkg").await.unwrap();
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
    async fn gpkg_geom() {
        let items = gpkg_items("../data/ne_extracts.gpkg", "ne_10m_lakes")
            .await
            .unwrap();
        assert_eq!(items.len(), 1355);
    }
}

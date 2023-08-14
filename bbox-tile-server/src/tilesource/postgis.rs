use crate::config::{PostgisSourceParamsCfg, VectorLayerCfg};
use crate::service::{TileService, TileSourceProviderConfigs};
use crate::tilesource::{
    mvt::MvtBuilder,
    postgis_queries::{QueryParam, SqlQuery},
    wms_fcgi::WmsMetrics,
    LayerInfo, SourceType, TileRead, TileResponse, TileSourceError, TileSourceProviderCfg,
};
use async_trait::async_trait;
use bbox_core::config::error_exit;
use bbox_core::pg_ds::PgDatasource;
use futures::TryStreamExt;
use geozero::{mvt, wkb, ToMvt};
use log::{debug, error, info, warn};
use sqlx::{
    postgres::{PgColumn, PgRow, PgStatement, PgTypeInfo},
    Column, Executor, Row, Statement, TypeInfo,
};
use std::collections::HashMap;
use std::io::Cursor;
use tile_grid::{Tms, Xyz};
use tilejson::{tilejson, TileJSON};

#[derive(Clone, Debug)]
pub struct PgSource {
    ds: PgDatasource,
    layers: HashMap<String, PgMvtLayer>, // t-rex uses BTreeMap
}

#[derive(Clone, Debug)]
pub struct PgMvtLayer {
    fields: Vec<FieldInfo>,
    geometry_field: String,
    geometry_type: Option<String>,
    fid_field: Option<String>,
    query_limit: Option<u32>,
    // TileJSON metadata
    // description: Option<String>,
    // maxzoom: Option<u8>,
    // minzoom: Option<u8>,
    /// Queries for all zoom levels
    queries: HashMap<u8, QueryInfo>,
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: String,
    pub info: FieldTypeInfo,
}

#[derive(Clone, PartialEq, Debug)]
pub enum FieldTypeInfo {
    Property(PgTypeInfo),
    Geometry,
    Ignored,
}

#[derive(Clone, Debug)]
struct QueryInfo {
    stmt: PgStatement<'static>,
    params: Vec<QueryParam>,
}

impl PgSource {
    pub async fn from_config(
        cfg: &PostgisSourceParamsCfg,
        sources: &TileSourceProviderConfigs,
        tms: &Tms,
    ) -> PgSource {
        let source_name = cfg.datasource.clone().unwrap_or("TODO".to_string());
        let TileSourceProviderCfg::Postgis(source) = sources.get(&source_name)
            .unwrap_or_else(|| error_exit(TileSourceError::TileSourceNotFound(source_name.to_string())))
        else { error_exit(TileSourceError::TileSourceTypeError("postgis".to_string())) };
        debug!("Connecting to PostGIS DB {}", &source.url);
        let ds = PgDatasource::new_pool(&source.url)
            .await
            .unwrap_or_else(error_exit); // TODO: better message
        let grid_srid = tms.crs().as_srid();

        let mut layers = HashMap::new();
        for layer in &cfg.layers {
            if let Ok(mvt_layer) = Self::setup_layer(&ds, layer, grid_srid).await {
                layers.insert(layer.name.clone(), mvt_layer);
            }
        }
        PgSource { ds, layers }
    }
    async fn setup_layer(
        ds: &PgDatasource,
        layer: &VectorLayerCfg,
        grid_srid: i32,
    ) -> Result<PgMvtLayer, TileSourceError> {
        // Configuration checks (TODO: add config_check to trait)
        if layer.query.len() == 0 && layer.table_name.is_none() {
            error!("Layer '{}': table_name undefined", layer.name);
            return Err(TileSourceError::TypeDetectionError);
        }

        let mut fields = Vec::new();
        let mut geometry_field = None;

        let zoom_steps = layer.zoom_steps();
        for zoom in &zoom_steps {
            let layer_query = layer.query(*zoom);
            let field_query = SqlQuery::build_field_query(layer, layer_query);
            fields = Vec::new(); // TODO: check consistency in all zoom levels
            match ds.pool.prepare(&field_query.sql).await {
                Ok(stmt) => {
                    for col in stmt.columns() {
                        let info = column_info(&col);
                        if let Some(geom_col) = &layer.geometry_field {
                            if col.name() == geom_col && info != FieldTypeInfo::Geometry {
                                error!(
                                    "Unsupported geometry type in layer {} at zoom level {zoom}",
                                    layer.name
                                );
                                continue;
                            }
                        } else if info == FieldTypeInfo::Geometry && geometry_field.is_none() {
                            // Default: use first geometry column
                            geometry_field = Some(col.name().to_string());
                        }
                        if info != FieldTypeInfo::Ignored {
                            let field_info = FieldInfo {
                                name: col.name().to_string(),
                                info,
                            };
                            fields.push(field_info);
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Field detection failed for layer {} at zoom level {zoom} - {e}",
                        layer.name
                    );
                    error!("Query: {}", field_query.sql);
                    return Err(TileSourceError::TypeDetectionError);
                }
            };
        }
        let Some(geometry_field) = geometry_field else {
            error!("No geometry column found in layer {}", layer.name);
            return Err(TileSourceError::TypeDetectionError);
        };
        let geom_name = layer.geometry_field.as_ref().unwrap_or(&geometry_field);

        let mut layer_queries = HashMap::new();
        for zoom in layer.minzoom()..=layer.maxzoom(22) {
            let layer_query = layer.query(zoom);
            let query =
                SqlQuery::build_tile_query(layer, geom_name, &fields, grid_srid, zoom, layer_query);
            let stmt = match ds.pool.prepare(&query.sql).await {
                Ok(stmt) => Statement::to_owned(&stmt), //stmt.to_owned()
                Err(e) => {
                    error!(
                        "Invalid query for layer {} at zoom level {zoom} - {e}",
                        layer.name
                    );
                    error!("Query: {}", query.sql);
                    return Err(TileSourceError::TypeDetectionError);
                }
            };
            if zoom_steps.contains(&zoom) {
                debug!("Query for minzoom {zoom}: {}", query.sql);
            }
            let query_info = QueryInfo {
                stmt,
                params: query.params.clone(),
            };
            layer_queries.insert(zoom, query_info);
        }

        Ok(PgMvtLayer {
            fields,
            geometry_field: geom_name.to_string(),
            geometry_type: layer.geometry_type.clone(),
            fid_field: layer.fid_field.clone(),
            query_limit: layer.query_limit.clone(),
            queries: layer_queries,
        })
    }
}

#[async_trait]
impl TileRead for PgSource {
    async fn xyz_request(
        &self,
        service: &TileService,
        tms_id: &str,
        tile: &Xyz,
        _format: &str,
        _scheme: &str,
        _host: &str,
        _req_path: &str,
        _metrics: &WmsMetrics,
    ) -> Result<TileResponse, TileSourceError> {
        let grid = service.grid(tms_id)?;
        let extent_info = service.xyz_extent(tms_id, tile)?;
        let extent = extent_info.extent;
        let mut mvt = MvtBuilder::new();
        for (id, layer) in &self.layers {
            let Some(query_info) = layer.queries.get(&tile.z) else {
                continue
            };
            let mut query = query_info.stmt.query();
            for param in &query_info.params {
                query = match param {
                    &QueryParam::Bbox => query
                        .bind(extent.left)
                        .bind(extent.bottom)
                        .bind(extent.right)
                        .bind(extent.top),
                    &QueryParam::Zoom => query.bind(tile.z as i32),
                    &QueryParam::PixelWidth => {
                        if let Some(pixel_width) = grid.resolution_z(tile.z) {
                            // correct: * 256.0 / layer.tile_size as f64
                            query.bind(pixel_width)
                        } else {
                            query
                        }
                    }
                    &QueryParam::ScaleDenominator => {
                        if let Some(m) = grid.matrix_z(tile.z) {
                            query.bind(m.scale_denominator)
                        } else {
                            query
                        }
                    }
                }
            }
            debug!("Query tile with {extent:?}");
            let mut rows = query.fetch(&self.ds.pool);
            let mut mvt_layer = MvtBuilder::new_layer(&id);
            let tile_size = mvt_layer.extent.unwrap_or(4096);
            let mut cnt = 0;
            let query_limit = layer.query_limit.unwrap_or(0);
            while let Some(row) = rows.try_next().await? {
                let wkb: wkb::Ewkb = row.try_get(layer.geometry_field.as_str())?;
                let mut feat = wkb
                    .to_mvt(
                        tile_size,
                        extent.left,
                        extent.bottom,
                        extent.right,
                        extent.top,
                    )?
                    .clone();
                for field in &layer.fields {
                    if field.name == layer.geometry_field {
                        continue;
                    }
                    let val = column_value(&row, field)?;
                    if let Some(fid_field) = &layer.fid_field {
                        if &field.name == fid_field {
                            if let Some(val) = val.int_value {
                                feat.id = Some(u64::try_from(val)?);
                                continue;
                            }
                        }
                    }
                    mvt.add_feature_attribute(&field.name, val, &mut feat)?;
                }
                mvt_layer.features.push(feat);
                cnt += 1;
                if cnt == query_limit {
                    info!(
                        "Features of layer {id} limited to {cnt} (tile query_limit reached, zoom level {})",
                        tile.z
                    );
                    break;
                }
            }
            mvt.push_layer(mvt_layer);
        }
        let blob = mvt.to_blob()?;
        let content_type = Some("application/x-protobuf".to_string());
        let body = Box::new(Cursor::new(blob));
        Ok(TileResponse {
            content_type,
            headers: TileResponse::new_headers(),
            body,
        })
    }
    fn source_type(&self) -> SourceType {
        SourceType::Vector
    }
    async fn tilejson(&self) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.other.insert("format".to_string(), "pbf".into());
        // tj.minzoom = self.minzoom;
        // tj.maxzoom = self.maxzoom;
        // tj.bounds = self.bounds;
        let layers = self
            .layers
            .iter()
            .map(|(id, layer)| {
                let fields = layer
                    .fields
                    .iter()
                    .filter(|field| {
                        if let FieldTypeInfo::Property(_) = &field.info {
                            if let Some(fid_field) = &layer.fid_field {
                                if &field.name == fid_field {
                                    return false;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .map(|field| (field.name.clone(), "".to_string()))
                    .collect();
                tilejson::VectorLayer {
                    id: id.clone(),
                    fields,
                    description: None,
                    maxzoom: None,
                    minzoom: None,
                    other: HashMap::default(),
                }
            })
            .collect();
        tj.vector_layers = Some(layers);
        Ok(tj)
    }
    async fn layers(&self) -> Result<Vec<LayerInfo>, TileSourceError> {
        let layers = self
            .layers
            .iter()
            .map(|(id, layer)| LayerInfo {
                name: id.clone(),
                geometry_type: layer.geometry_type.clone(),
            })
            .collect();
        Ok(layers)
    }
}

fn column_info(col: &PgColumn) -> FieldTypeInfo {
    let pg_type = col.type_info().name();
    // Supported column types
    // https://github.com/launchbadge/sqlx/blob/d0fbe7f/sqlx-postgres/src/type_info.rs#L469
    if [
        "VARCHAR",
        "TEXT",
        "CHAR_ARRAY",
        "FLOAT4",
        "FLOAT8",
        "INT2",
        "INT4",
        "INT8",
        "BOOL",
    ]
    .contains(&pg_type)
    {
        FieldTypeInfo::Property(col.type_info().clone())
    } else if ["NUMERIC"].contains(&pg_type) {
        warn!(
            "Converting column `{}` with type `{}` to supported type",
            col.type_info(),
            col.name()
        );
        FieldTypeInfo::Property(col.type_info().clone())
    } else if ["geometry", "geography"].contains(&pg_type) {
        FieldTypeInfo::Geometry
    } else {
        warn!(
            "Type `{}` of column `{}` not supported",
            col.type_info(),
            col.name()
        );
        FieldTypeInfo::Ignored
    }
}

/// Convert PG column value to MVT value
fn column_value(row: &PgRow, field: &FieldInfo) -> Result<mvt::tile::Value, sqlx::Error> {
    let mut mvt_val = mvt::tile::Value::default();
    let FieldTypeInfo::Property(pg_type) = &field.info  else {
        return Ok(mvt_val) // Warning or error?
    };
    let col = field.name.as_str();
    match pg_type.name() {
        "VARCHAR" | "TEXT" | "CHAR_ARRAY" => {
            mvt_val.string_value = Some(row.try_get::<String, _>(col)?);
            // or: mvt::tile::Value { string_value: Some(col_val), ..Default::default() }
        }
        "FLOAT4" => {
            mvt_val.float_value = Some(row.try_get::<f32, _>(col)?);
        }
        "FLOAT8" => {
            mvt_val.double_value = Some(row.try_get::<f64, _>(col)?);
        }
        "INT2" => {
            mvt_val.int_value = Some(row.try_get::<i16, _>(col)?.into());
        }
        "INT4" => {
            mvt_val.int_value = Some(row.try_get::<i32, _>(col)?.into());
        }
        "INT8" => {
            mvt_val.int_value = Some(row.try_get::<i64, _>(col)?);
        }
        "BOOL" => {
            mvt_val.bool_value = Some(row.try_get::<bool, _>(col)?);
        }
        _ => {}
    }
    Ok(mvt_val)
}
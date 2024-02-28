//! PostGIS tile source.

use crate::config::{PostgisSourceParamsCfg, VectorLayerCfg};
use crate::datasource::{
    mvt::MvtBuilder,
    postgis_queries::{QueryParam, SqlQuery},
    wms_fcgi::HttpRequestParams,
    LayerInfo, SourceType, TileRead, TileResponse, TileSourceError,
};
use crate::service::TileService;
use async_trait::async_trait;
use bbox_core::pg_ds::PgDatasource;
use bbox_core::Format;
use futures::TryStreamExt;
use geozero::{mvt, wkb, ToMvt};
use log::{debug, error, info, warn};
use serde_json::json;
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
    diagnostics: bool,
}

#[derive(Clone, Debug)]
pub struct PgMvtLayer {
    fields: Vec<FieldInfo>,
    geometry_field: String,
    geometry_type: Option<String>,
    /// ST_AsMvt returns geometries in tile coordinate system
    tile_coord_sys: bool,
    tile_size: u32,
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

pub type Datasource = PgDatasource;

impl PgSource {
    pub async fn create(ds: &PgDatasource, cfg: &PostgisSourceParamsCfg, tms: &Tms) -> PgSource {
        let grid_srid = tms.crs().as_srid();

        let mut layers = HashMap::new();
        for layer in &cfg.layers {
            match Self::setup_layer(ds, layer, grid_srid, cfg.postgis2).await {
                Ok(mvt_layer) => {
                    layers.insert(layer.name.clone(), mvt_layer);
                }
                Err(_) => {
                    error!("Layer `{}`: skipping", layer.name)
                }
            };
        }
        PgSource {
            ds: ds.clone(),
            layers,
            diagnostics: cfg.diagnostics,
        }
    }
    async fn setup_layer(
        ds: &PgDatasource,
        layer: &VectorLayerCfg,
        grid_srid: i32,
        postgis2: bool,
    ) -> Result<PgMvtLayer, TileSourceError> {
        // Configuration checks (TODO: add config_check to trait)
        if layer.queries.is_empty() && layer.table_name.is_none() {
            error!("Layer '{}': table_name undefined", layer.name);
            return Err(TileSourceError::TypeDetectionError);
        }

        let mut all_fields: HashMap<u8, Vec<FieldInfo>> = HashMap::new();
        let mut geometry_field = None;

        let zoom_steps = layer.zoom_steps();
        for zoom in &zoom_steps {
            let layer_query = layer.query(*zoom);
            let field_query = SqlQuery::build_field_query(layer, layer_query);
            let mut fields = Vec::new();
            match ds.pool.prepare(&field_query.sql).await {
                Ok(stmt) => {
                    for col in stmt.columns() {
                        let info = column_info(col, &layer.name);
                        if let Some(geom_col) = &layer.geometry_field {
                            if col.name() == geom_col {
                                if info == FieldTypeInfo::Geometry {
                                    geometry_field = Some(geom_col.to_string());
                                } else {
                                    error!(
                                        "Layer `{}`: Unsupported geometry type at zoom level {zoom}",
                                        layer.name
                                    );
                                    continue;
                                }
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
                        "Layer `{}`: Field detection failed at zoom level {zoom} - {e}",
                        layer.name
                    );
                    error!("Query: {}", field_query.sql);
                    return Err(TileSourceError::TypeDetectionError);
                }
            };
            all_fields.insert(*zoom, fields);
        }
        let Some(geometry_field) = geometry_field else {
            // TODO: check for valid geometry_field in *all* zoom levels
            error!("Layer `{}`: No geometry column found", layer.name);
            return Err(TileSourceError::TypeDetectionError);
        };
        let geom_name = layer.geometry_field.as_ref().unwrap_or(&geometry_field);

        let mut layer_queries = HashMap::new();
        for zoom in layer.minzoom()..=layer.maxzoom(22) {
            let layer_query = layer.query(zoom);
            let fields =
                VectorLayerCfg::zoom_step_entry(&all_fields, zoom).expect("invalid zoom steps");
            let query = SqlQuery::build_tile_query(
                layer,
                geom_name,
                fields,
                grid_srid,
                zoom,
                layer_query,
                postgis2,
            );
            let stmt = match ds.pool.prepare(&query.sql).await {
                Ok(stmt) => Statement::to_owned(&stmt), //stmt.to_owned()
                Err(e) => {
                    error!(
                        "Layer `{}`: Invalid query at zoom level {zoom} - {e}",
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

        // collect fields from all zoom step levels
        let fields = all_fields
            .into_values()
            .flatten()
            .map(|f| (f.name.clone(), f))
            .collect::<HashMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>();

        Ok(PgMvtLayer {
            fields,
            geometry_field: geom_name.to_string(),
            geometry_type: layer.geometry_type.clone(),
            tile_coord_sys: !postgis2,
            tile_size: layer.tile_size,
            fid_field: layer.fid_field.clone(),
            query_limit: layer.query_limit,
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
        _format: &Format,
        _request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        let grid = service.grid(tms_id)?;
        let extent_info = service.xyz_extent(tms_id, tile)?;
        let extent = &extent_info.extent;
        let mut mvt = MvtBuilder::new();
        for (id, layer) in &self.layers {
            let Some(query_info) = layer.queries.get(&tile.z) else {
                continue;
            };
            let mut query = query_info.stmt.query();
            for param in &query_info.params {
                query = match *param {
                    QueryParam::Bbox => query
                        .bind(extent.left)
                        .bind(extent.bottom)
                        .bind(extent.right)
                        .bind(extent.top),
                    QueryParam::Zoom => query.bind(tile.z as i32),
                    QueryParam::PixelWidth => {
                        if let Some(pixel_width) = grid.resolution_z(tile.z) {
                            // TODO: grid_width = grid.tile_width_z(tile.z)
                            let grid_width: u16 =
                                grid.tms.tile_matrices[tile.z as usize].tile_width.into();
                            let mvt_pixel_width =
                                pixel_width * grid_width as f64 / layer.tile_size as f64;
                            query.bind(mvt_pixel_width)
                        } else {
                            query
                        }
                    }
                    QueryParam::ScaleDenominator => {
                        if let Some(m) = grid.matrix_z(tile.z) {
                            query.bind(m.scale_denominator)
                        } else {
                            query
                        }
                    }
                }
            }
            debug!(
                "Query tile {}/{}/{} with {extent:?}",
                tile.z, tile.x, tile.y
            );
            let mut rows = query.fetch(&self.ds.pool);
            let mut mvt_layer = MvtBuilder::new_layer(id, layer.tile_size);
            let mut cnt = 0;
            let query_limit = layer.query_limit.unwrap_or(0);
            while let Some(row) = rows.try_next().await? {
                let Some(wkb) =
                    row.try_get::<Option<wkb::Ewkb>, _>(layer.geometry_field.as_str())?
                else {
                    // Skip NULL geometries
                    continue;
                };
                let mut feat = if layer.tile_coord_sys {
                    wkb.to_mvt_unscaled()?
                } else {
                    wkb.to_mvt(
                        layer.tile_size,
                        extent.left,
                        extent.bottom,
                        extent.right,
                        extent.top,
                    )?
                };
                for field in &layer.fields {
                    if field.name == layer.geometry_field {
                        continue;
                    }
                    if let Some(val) = column_value(&row, field)? {
                        if let Some(fid_field) = &layer.fid_field {
                            if &field.name == fid_field {
                                if let Some(val) = val.int_value {
                                    feat.id = Some(u64::try_from(val)?);
                                    continue;
                                }
                            }
                        }
                        mvt_layer.add_feature_attribute(&mut feat, &field.name, val)?;
                    } // skip null values
                }
                mvt_layer.push_feature(feat);
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
        if self.diagnostics {
            mvt.add_diagnostics_layer(tile, &extent_info)?;
        }
        let blob = mvt.into_blob()?;
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
    async fn tilejson(&self, format: &Format) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.other
            .insert("format".to_string(), format.file_suffix().into());
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
        let mut layers: Vec<LayerInfo> = self
            .layers
            .iter()
            .map(|(id, layer)| LayerInfo {
                name: id.clone(),
                geometry_type: layer.geometry_type.clone(),
                style: None,
            })
            .collect();
        if self.diagnostics {
            layers.push(LayerInfo {
                name: "diagnostics-tile".to_string(),
                geometry_type: Some("line".to_string()),
                style: None,
            });
            layers.push(LayerInfo {
                name: "diagnostics-label".to_string(),
                geometry_type: Some("symbol".to_string()),
                style: Some(json!({"layout": {"text-field": "{zxy}"}})),
            });
        }
        Ok(layers)
    }
}

fn column_info(col: &PgColumn, layer_name: &str) -> FieldTypeInfo {
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
            "Layer `{layer_name}`: Converting column `{}` with type `{}` to supported type",
            col.name(),
            col.type_info()
        );
        FieldTypeInfo::Property(col.type_info().clone())
    } else if ["geometry", "geography"].contains(&pg_type) {
        FieldTypeInfo::Geometry
    } else {
        warn!(
            "Layer `{layer_name}`: Type `{}` of column `{}` not supported",
            col.type_info(),
            col.name()
        );
        FieldTypeInfo::Ignored
    }
}

/// Convert PG column value to MVT value
fn column_value(row: &PgRow, field: &FieldInfo) -> Result<Option<mvt::tile::Value>, sqlx::Error> {
    let FieldTypeInfo::Property(pg_type) = &field.info else {
        return Ok(None); // Warning or error?
    };
    let col = field.name.as_str();
    let mut mvt_val = mvt::tile::Value::default();
    match pg_type.name() {
        "VARCHAR" | "TEXT" | "CHAR_ARRAY" => {
            mvt_val.string_value = row.try_get::<Option<String>, _>(col)?;
            // or: mvt::tile::Value { string_value: Some(col_val), ..Default::default() }
        }
        "FLOAT4" => {
            mvt_val.float_value = row.try_get::<Option<f32>, _>(col)?;
        }
        "FLOAT8" => {
            mvt_val.double_value = row.try_get::<Option<f64>, _>(col)?;
        }
        "INT2" => {
            mvt_val.int_value = row.try_get::<Option<i16>, _>(col)?.map(i16::into);
        }
        "INT4" => {
            mvt_val.int_value = row.try_get::<Option<i32>, _>(col)?.map(i32::into);
        }
        "INT8" => {
            mvt_val.int_value = row.try_get::<Option<i64>, _>(col)?;
        }
        "BOOL" => {
            mvt_val.bool_value = row.try_get::<Option<bool>, _>(col)?;
        }
        _ => {}
    }
    if mvt_val == mvt::tile::Value::default() {
        // TODO: check optimization (compare with static?)
        Ok(None)
    } else {
        Ok(Some(mvt_val))
    }
}

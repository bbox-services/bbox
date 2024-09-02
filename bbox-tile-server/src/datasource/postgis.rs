//! PostGIS tile source.

use crate::config::{PostgisSourceParamsCfg, TilesetTmsCfg, VectorLayerCfg};
use crate::datasource::{
    mvt::MvtBuilder,
    postgis_queries::{QueryParam, SqlQuery},
    wms_fcgi::HttpRequestParams,
    LayerInfo, SourceType, TileSource, TileSourceError,
};
use crate::filter_params::FilterParams;
use crate::service::{TileSetGrid, TmsExtensions};
use async_trait::async_trait;
use bbox_core::pg_ds::PgDatasource;
use bbox_core::{Format, TileResponse};
use futures::TryStreamExt;
use geozero::{mvt, wkb, ToMvt};
use log::{debug, error, info, warn};
use serde_json::json;
use sqlx::{
    postgres::{PgColumn, PgRow, PgStatement, PgTypeInfo},
    Column, Executor, Row, Statement, TypeInfo,
};
use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use tile_grid::{BoundingBox, Tms, Xyz};
use tilejson::{tilejson, TileJSON};

#[derive(Clone, Debug)]
pub struct PgSource {
    ds: PgDatasource,
    layers: BTreeMap<String, PgMvtLayer>,
    /// Config with TileJSON metadata
    config: PostgisSourceParamsCfg,
}

#[derive(Clone, Debug)]
pub struct PgMvtLayer {
    geometry_type: Option<String>,
    /// ST_AsMvt returns geometries in tile coordinate system
    tile_coord_sys: bool,
    tile_size: u32,
    fid_field: Option<String>,
    query_limit: Option<u32>,
    /// Queries for zoom steps for each grid_srid
    queries: HashMap<i32, HashMap<u8, QueryInfo>>,
    /// Query zoom step for all zoom levels (z -> minzoom step)
    query_zoom_steps: HashMap<u8, u8>,
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
    geometry_field: String,
    fields: Vec<FieldInfo>,
}

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub name: String,
    pub info: FieldTypeInfo,
}

pub type Datasource = PgDatasource;

impl PgMvtLayer {
    /// Get query for zoom level
    fn query(&self, grid_srid: i32, zoom: u8) -> Option<&QueryInfo> {
        self.query_zoom_steps
            .get(&zoom)
            .and_then(|minzoom| self.queries.get(&grid_srid).and_then(|gq| gq.get(minzoom)))
    }
    pub fn minzoom(&self) -> u8 {
        *self.query_zoom_steps.keys().min().unwrap_or(&0)
    }
    pub fn maxzoom(&self) -> u8 {
        *self.query_zoom_steps.keys().max().unwrap_or(&0)
    }
}

impl PgSource {
    pub async fn create(
        ds: &PgDatasource,
        cfg: &PostgisSourceParamsCfg,
        ts_grids: &[TileSetGrid],
        tms_cfg: &[TilesetTmsCfg],
    ) -> PgSource {
        let mut layers = BTreeMap::new();
        for layer in &cfg.layers {
            match Self::setup_layer(ds, layer, ts_grids, tms_cfg, cfg.postgis2).await {
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
            config: cfg.clone(),
        }
    }
    async fn setup_layer(
        ds: &PgDatasource,
        layer: &VectorLayerCfg,
        ts_grids: &[TileSetGrid],
        tms_cfg: &[TilesetTmsCfg],
        postgis2: bool,
    ) -> Result<PgMvtLayer, TileSourceError> {
        // Configuration checks (TODO: add config_check to trait)
        if layer.queries.is_empty() && layer.table_name.is_none() {
            error!("Layer '{}': table_name undefined", layer.name);
            return Err(TileSourceError::TypeDetectionError);
        }

        fn tile_srid_z(ts_grids: &[TileSetGrid], zoom: u8) -> Option<i32> {
            ts_grids
                .iter()
                .rev()
                .find(|entry| entry.minzoom <= zoom && entry.maxzoom >= zoom)
                .map(|entry| entry.tms.srid())
        }

        let zoom_steps = layer.zoom_steps(tms_cfg);
        if zoom_steps.len() > 1 {
            debug!("Layer `{}` zoom steps: {:?}", layer.name, zoom_steps);
        }
        let mut layer_queries = HashMap::new();
        for grid in ts_grids {
            for zs in &zoom_steps {
                let zoom = *zs;
                let layer_query = layer.query(zoom);
                let tile_srid = tile_srid_z(ts_grids, zoom).unwrap_or(grid.tms.srid());
                let field_query = SqlQuery::build_field_query(layer, layer_query);
                let param_types = field_query.param_types();
                let mut geometry_field = None;
                let mut fields = Vec::new();
                match ds.pool.prepare_with(&field_query.sql, &param_types).await {
                    Ok(stmt) => {
                        for col in stmt.columns() {
                            let info = column_info(col, &layer.name);
                            if let Some(geom_col) = &layer.geometry_field {
                                if col.name() == geom_col {
                                    if info == FieldTypeInfo::Geometry {
                                        geometry_field = Some(geom_col.to_string());
                                    } else {
                                        error!(
                                            "Layer `{}` z{zoom}: Unsupported geometry type",
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
                        debug!("Query parameters: {:?}", stmt.parameters());
                    }
                    Err(e) => {
                        error!(
                            "Layer `{}` z{zoom}: Field detection failed - {e}",
                            layer.name
                        );
                        error!(" Query: {}", field_query.sql);
                        return Err(TileSourceError::TypeDetectionError);
                    }
                };
                let Some(geometry_field) = geometry_field else {
                    error!("Layer `{}`: No geometry column found", layer.name);
                    return Err(TileSourceError::TypeDetectionError);
                };
                let geom_name = layer.geometry_field.as_ref().unwrap_or(&geometry_field);
                let query = SqlQuery::build_tile_query(
                    layer,
                    geom_name,
                    &fields,
                    tile_srid,
                    zoom,
                    layer_query,
                    postgis2,
                );
                let param_types = query.param_types();
                let stmt = match ds.pool.prepare_with(&query.sql, &param_types).await {
                    Ok(stmt) => Statement::to_owned(&stmt), //stmt.to_owned()
                    Err(e) => {
                        error!("Layer `{}` z{zoom}: Invalid query - {e}", layer.name);
                        error!(" Query: {}", query.sql);
                        return Err(TileSourceError::TypeDetectionError);
                    }
                };
                // Workaround for cached queries with incorrect parameter types
                // for _ in 0..ds.pool.size() {
                //     ds.pool.acquire().await?.clear_cached_statements().await?;
                // }
                debug!(
                    "Layer `{}`: Query for minzoom {zoom}: {}",
                    layer.name, query.sql
                );
                let query_info = QueryInfo {
                    stmt,
                    params: query.params.clone(),
                    fields: fields.clone(),
                    geometry_field: geometry_field.clone(),
                };
                layer_queries
                    .entry(tile_srid)
                    .or_insert(HashMap::new())
                    .insert(zoom, query_info);
            }
        }

        // Lookup table for all zoom levels
        let zoom_steps = layer.zoom_steps(tms_cfg);
        let maxzoom = ts_grids
            .iter()
            .map(|g| g.tms.maxzoom())
            .max()
            .expect("default grid missing");
        let mut query_zoom_steps = HashMap::new();
        for zoom in layer.minzoom()..=layer.maxzoom(maxzoom) {
            let z = zoom_steps
                .iter()
                .rev()
                .find(|z| zoom >= **z)
                .expect("invalid zoom steps");
            query_zoom_steps.insert(zoom, *z);
        }

        Ok(PgMvtLayer {
            geometry_type: layer.geometry_type.clone(),
            tile_coord_sys: !postgis2,
            tile_size: layer.tile_size,
            fid_field: layer.fid_field.clone(),
            query_limit: layer.query_limit,
            queries: layer_queries,
            query_zoom_steps,
        })
    }
}

fn layer_query<'a>(
    layer: &'a PgMvtLayer,
    query_info: &'a QueryInfo,
    tile: &Xyz,
    grid: &Tms,
    extent: &BoundingBox,
    filter: &'a FilterParams,
) -> Result<sqlx::query::Query<'a, sqlx::Postgres, sqlx::postgres::PgArguments>, TileSourceError> {
    let mut query = query_info.stmt.query();
    for param in &query_info.params {
        query = match *param {
            QueryParam::Bbox => query
                .bind(extent.left)
                .bind(extent.bottom)
                .bind(extent.right)
                .bind(extent.top),
            QueryParam::Zoom => query.bind(tile.z as i32),
            QueryParam::X => query.bind(tile.x as i32),
            QueryParam::Y => query.bind(tile.y as i32),
            QueryParam::PixelWidth => {
                if let Some(pixel_width) = grid.resolution_z(tile.z) {
                    // TODO: grid_width = grid.tile_width_z(tile.z)
                    let grid_width: u16 = grid.tms.tile_matrices[tile.z as usize].tile_width.into();
                    let mvt_pixel_width = pixel_width * grid_width as f64 / layer.tile_size as f64;
                    query.bind(mvt_pixel_width)
                } else {
                    info!("Undefined resolution for z={}", tile.z);
                    return Err(TileSourceError::TileXyzError);
                }
            }
            QueryParam::ScaleDenominator => {
                if let Some(m) = grid.matrix_z(tile.z) {
                    query.bind(m.scale_denominator)
                } else {
                    info!("Undefined scale_denominator for z={}", tile.z);
                    return Err(TileSourceError::FilterParamError);
                }
            }
            QueryParam::QueryField(ref field) => {
                if let Some(value) = filter.filters.get(field) {
                    query.bind(value)
                } else {
                    info!("Filter parameter `{field}` missing");
                    return Err(TileSourceError::FilterParamError);
                }
            }
        }
    }
    Ok(query)
}

#[async_trait]
impl TileSource for PgSource {
    async fn xyz_request(
        &self,
        tms: &Tms,
        tile: &Xyz,
        filter: &FilterParams,
        _format: &Format,
        _request_params: HttpRequestParams<'_>,
    ) -> Result<TileResponse, TileSourceError> {
        let extent_info = tms.xyz_extent(tile)?;
        let extent = &extent_info.extent;
        debug!(
            "Query tile {}/{}/{} with {extent:?}",
            tile.z, tile.x, tile.y
        );
        let tile_srid = tms.srid();
        let mut mvt = MvtBuilder::new();
        for (id, layer) in &self.layers {
            let Some(query_info) = layer.query(tile_srid, tile.z) else {
                continue;
            };
            let query = layer_query(layer, query_info, tile, tms, extent, filter)?;
            debug!("Query layer `{id}`");
            let mut rows = query.fetch(&self.ds.pool);
            let mut mvt_layer = MvtBuilder::new_layer(id, layer.tile_size);
            let mut cnt = 0;
            let query_limit = layer.query_limit.unwrap_or(0);
            while let Some(row) = rows.try_next().await? {
                let Some(wkb) =
                    row.try_get::<Option<wkb::Ewkb>, _>(query_info.geometry_field.as_str())?
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
                for field in &query_info.fields {
                    if field.name == query_info.geometry_field {
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
                        "Layer `{id}`: Features limited to {cnt} (tile query_limit reached, zoom level {})",
                        tile.z
                    );
                    break;
                }
            }
            mvt.push_layer(mvt_layer);
        }
        if let Some(diaganostics_cfg) = &self.config.diagnostics {
            mvt.add_diagnostics_layer(diaganostics_cfg, tile, &extent_info)?;
        }
        let blob = mvt.into_blob()?;
        let mut response = TileResponse::new();
        response.set_content_type("application/x-protobuf");
        let body = Box::new(Cursor::new(blob));
        Ok(response.with_body(body))
    }
    fn source_type(&self) -> SourceType {
        SourceType::Vector
    }
    async fn tilejson(&self, tms: &Tms, format: &Format) -> Result<TileJSON, TileSourceError> {
        let mut tj = tilejson! { tiles: vec![] };
        tj.attribution = Some(self.config.attribution());
        // Minimum zoom level for which tiles are available.
        // Optional. Default: 0. >= 0, <= 30.
        tj.minzoom = Some(tms.minzoom());
        // Maximum zoom level for which tiles are available.
        // Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
        // Optional. Default: 30. >= 0, <= 30. (Mapbox Style default: 22)
        tj.maxzoom = Some(tms.maxzoom());
        let extent = self.config.get_extent();
        tj.bounds = Some(tilejson::Bounds {
            left: extent.minx,
            bottom: extent.miny,
            right: extent.maxx,
            top: extent.maxy,
        });
        let center = self.config.get_center();
        tj.center = Some(tilejson::Center {
            longitude: center.1,
            latitude: center.0,
            zoom: self.config.get_start_zoom(),
        });
        tj.other
            .insert("format".to_string(), format.file_suffix().into());

        let grid_srid = tms.srid();
        if grid_srid != 3857 {
            // TODO: add full grid information according to GDAL extension
            // https://github.com/OSGeo/gdal/blob/release/3.4/gdal/ogr/ogrsf_frmts/mvt/ogrmvtdataset.cpp#L5497
            tj.other
                .insert("srs".to_string(), tms.crs().as_known_crs().into());
        }
        // TODO: advertise zoom level specific srids
        let mut layers: Vec<tilejson::VectorLayer> = self
            .layers
            .iter()
            .map(|(id, layer)| {
                // Collected fields from all zoom step levels
                let fields = layer
                    .queries
                    .values()
                    .flatten()
                    .flat_map(|(_z, q)| q.fields.clone())
                    .map(|f| (f.name.clone(), f))
                    .collect::<HashMap<_, _>>()
                    .values()
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
                    minzoom: Some(layer.minzoom()),
                    maxzoom: Some(layer.maxzoom()),
                    other: BTreeMap::default(),
                }
            })
            .collect();
        if self.config.diagnostics.is_some() {
            layers.push(tilejson::VectorLayer {
                id: "diagnostics-tile".to_string(),
                fields: BTreeMap::from([
                    (
                        "layer-total-percent".to_string(),
                        "Total size in bytes (uncompressed)".to_string(),
                    ),
                    (
                        "layer-total-percent".to_string(),
                        "Total size relative to reference size".to_string(),
                    ),
                ]),
                description: None,
                maxzoom: None,
                minzoom: None,
                other: BTreeMap::default(),
            });
            layers.push(tilejson::VectorLayer {
                id: "diagnostics-label".to_string(),
                fields: BTreeMap::from([
                    ("zxy".to_string(), "tile number".to_string()),
                    ("tile-top".to_string(), "tile extent".to_string()),
                    ("tile-left".to_string(), "tile extent".to_string()),
                    ("tile-bottom".to_string(), "tile extent".to_string()),
                    ("tile-right".to_string(), "tile extent".to_string()),
                ]),
                description: None,
                maxzoom: None,
                minzoom: None,
                other: BTreeMap::default(),
            });
        }
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
        if self.config.diagnostics.is_some() {
            layers.push(LayerInfo {
                name: "diagnostics-tile".to_string(),
                geometry_type: Some("line".to_string()),
                style: Some(json!({"paint": {
                  "line-color": "rgba(196, 43, 43, 0.81)",
                  "line-width": [
                    "interpolate",
                    ["linear"],
                    ["get", "layer-total-percent"],
                    0, 1,
                    100, 50
                  ],
                }})),
            });
            layers.push(LayerInfo {
                name: "diagnostics-label".to_string(),
                geometry_type: Some("symbol".to_string()),
                style: Some(json!({
                  "layout": {"text-field": "{zxy}"},
                  "paint": {
                    "text-color": "rgba(196, 43, 43, 1)",
                    "text-halo-width": 2,
                    "text-halo-color": "rgba(255, 255, 255, 1)"
                  }
                })),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VectorLayerQueryCfg;
    use bbox_core::config::DsPostgisCfg;
    use bbox_core::pg_ds::PgDatasource;
    use test_log::test;
    use tile_grid::tms;

    // docker run -p 127.0.0.1:5439:5432 -d --name mvtbenchdb --rm sourcepole/mvtbenchdb:v1.2
    //
    // For debug log output run with:
    // RUST_LOG=debug cargo test -- --ignored --nocapture

    async fn pg_source(query: Option<String>) -> PgSource {
        let ds_cfg = DsPostgisCfg {
            url: "postgresql://mvtbench:mvtbench@127.0.0.1:5439/mvtbench".to_string(),
        };
        let mut queries = Vec::new();
        if let Some(sql) = query {
            queries.push(VectorLayerQueryCfg {
                minzoom: Some(0),
                maxzoom: None,
                simplify: None,
                tolerance: None,
                sql: Some(sql),
            });
        }
        let layer = VectorLayerCfg {
            name: "layer1".to_string(),
            geometry_field: Some("wkb_geometry".to_string()),
            geometry_type: None,
            srid: Some(3857),
            no_transform: false,
            fid_field: None,
            table_name: Some("ne_10m_rivers_lake_centerlines".to_string()),
            query_limit: None,
            queries,
            minzoom: None,
            maxzoom: None,
            tile_size: 4096,
            simplify: false,
            tolerance: "!pixel_width!/2".to_string(),
            buffer_size: Some(0),
            make_valid: false,
            shift_longitude: false,
        };
        let pg_src_cfg = PostgisSourceParamsCfg {
            datasource: None,
            extent: None,
            center: None,
            start_zoom: None,
            attribution: None,
            postgis2: false,
            diagnostics: None,
            layers: vec![layer],
        };
        let tms = tms().lookup("WebMercatorQuad").unwrap();
        let ts_grids = vec![TileSetGrid {
            tms,
            minzoom: 0,
            maxzoom: 24,
        }];
        let ds = PgDatasource::from_config(&ds_cfg, None).await.unwrap();
        PgSource::create(&ds, &pg_src_cfg, &ts_grids, &Vec::new()).await
    }

    #[test(tokio::test)]
    #[ignore]
    async fn tile_query() {
        let pg = pg_source(None).await;
        let layer = pg.layers.get("layer1").unwrap();
        let tms = tms().lookup("WebMercatorQuad").unwrap();
        let tile = Xyz::new(0, 0, 0);
        let query_info = layer.query(tms.srid(), tile.z).unwrap();
        let extent = tms.xy_bounds(&tile);
        let filter = FilterParams::default();
        let query = layer_query(layer, query_info, &tile, &tms, &extent, &filter).unwrap();
        let rows = query.fetch_all(&pg.ds.pool).await.unwrap();
        assert_eq!(rows.len(), 1473);
    }

    #[test(tokio::test)]
    #[ignore]
    async fn country_geoms() {
        // Trying to reproduce an empty ST_AsMvtGeom for 'BRA' with:
        // SELECT ST_Summary(ST_AsMvtGeom(wkb_geometry, ST_TileEnvelope(0, 0, 0), 4096, 0, true)) AS wkb_geometry,"adm0_a3","mapcolor7"
        //  FROM (SELECT wkb_geometry, adm0_a3, mapcolor7 FROM ne_10m_admin_0_countries WHERE sov_a3 IN ('BRA', 'ARG')) AS _q
        //  WHERE wkb_geometry && ST_TileEnvelope(0, 0, 0);
        // Works with same PostGIS version, but older GEOS, Proj, etc.
        // POSTGIS="3.4.2 c19ce56" [EXTENSION] PGSQL="160" GEOS="3.12.1-CAPI-1.18.1" PROJ="9.4.0 NETWORK_ENABLED=OFF URL_ENDPOINT=https://cdn.proj.org USER_WRITABLE_DIRECTORY=/var/lib/pgsql/.local/share/proj DATABASE_PATH=/usr/share/proj/proj.db" LIBXML="2.12.5" LIBJSON="0.17" LIBPROTOBUF="1.5.0" WAGYU="0.5.0 (Internal)"
        let pg = pg_source(Some("SELECT wkb_geometry, adm0_a3, mapcolor7 FROM ne_10m_admin_0_countries WHERE sov_a3 IN ('BRA', 'ARG')".to_string())).await;
        let layer = pg.layers.get("layer1").unwrap();
        let tms = tms().lookup("WebMercatorQuad").unwrap();
        let tile = Xyz::new(0, 0, 0);
        let query_info = layer.query(tms.srid(), tile.z).unwrap();
        let extent = tms.xy_bounds(&tile);
        let filter = FilterParams::default();
        let query = layer_query(layer, query_info, &tile, &tms, &extent, &filter).unwrap();
        let rows = query.fetch_all(&pg.ds.pool).await.unwrap();
        assert_eq!(rows.len(), 2);
        // rows.iter().for_each(|row| {
        //     dbg!(row.get::<&str, _>("adm0_a3"));
        // });
        let geoms = rows
            .iter()
            .map(|row| row.try_get::<Option<wkb::Ewkb>, _>("wkb_geometry").unwrap())
            .collect::<Vec<_>>();
        assert_eq!(geoms.len(), 2);
        assert!(geoms[0].is_some());
        assert!(geoms[1].is_some());
    }
}

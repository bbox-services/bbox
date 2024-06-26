use crate::config::VectorLayerCfg;
use crate::datasource::postgis::{FieldInfo, FieldTypeInfo};
use log::{info, warn};
use regex::Regex;
use sqlx::{postgres::PgTypeInfo, TypeInfo};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct SqlQuery {
    pub sql: String,
    pub params: Vec<QueryParam>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum QueryParam {
    Bbox,
    Zoom,
    X,
    Y,
    QueryField(String),
    PixelWidth,
    ScaleDenominator,
}

impl SqlQuery {
    /// Initial select query for column type detection
    pub fn build_field_query(layer: &VectorLayerCfg, user_query: Option<&String>) -> Self {
        if let Some(sql) = user_query {
            // Replace vars with valid SQL
            let bbox_expr = "ST_MakeEnvelope($1,$2,$3,$4,3857)";
            Self::replace_params(sql, bbox_expr.to_string(), bbox_expr.to_string())
        } else {
            let sql = format!(
                "SELECT * FROM {}",
                layer
                    .table_name
                    .as_ref()
                    .expect("query and table_name undefined")
            );
            SqlQuery {
                sql,
                params: Vec::new(),
            }
        }
    }

    /// Runtime query
    pub fn build_tile_query(
        layer: &VectorLayerCfg,
        geom_name: &str,
        data_columns: &[FieldInfo],
        tile_srid: i32,
        zoom: u8,
        user_query: Option<&String>,
        postgis2: bool,
    ) -> Self {
        let mut sqlquery;
        let geom_expr = if postgis2 {
            build_geom_expr_postgis2(layer, geom_name, tile_srid, zoom)
        } else {
            build_geom_expr(layer, geom_name, tile_srid, zoom)
        };
        let select_list = build_select_list(geom_expr, data_columns);
        let intersect_clause = format!(" WHERE {geom_name} && !bbox!");

        if let Some(user_query) = user_query {
            // user query
            sqlquery = format!("SELECT {select_list} FROM ({user_query}) AS _q");
            let is_xyz_query = user_query.contains("!x!")
                && user_query.contains("!y!")
                && user_query.contains("!zoom!");
            if !user_query.contains("!bbox!") && !is_xyz_query {
                sqlquery.push_str(&intersect_clause);
            }
        } else {
            // automatic query
            sqlquery = format!(
                "SELECT {select_list} FROM {}{intersect_clause}",
                layer
                    .table_name
                    .as_ref()
                    .expect("query and table_name undefined")
            );
        };

        let bbox_expr = build_bbox_expr(layer, tile_srid, layer.buffer_size);
        // !bbox_unbuffered! replacement expression for ST_AsMVTGeom
        let bbox_expr_unbuffered = format!("ST_MakeEnvelope($1,$2,$3,$4,{tile_srid})");
        Self::replace_params(&sqlquery, bbox_expr, bbox_expr_unbuffered)
    }

    /// Replace variables (!bbox!, !zoom!, etc.) in query
    // https://github.com/mapnik/mapnik/wiki/PostGIS
    fn replace_params(sqlin: &str, bbox_expr: String, bbox_expr_unbuffered: String) -> Self {
        let mut sql = sqlin.to_string();
        let mut params = Vec::new();
        let mut numvars = 0;

        if sql.contains("!bbox!") || sql.contains("!bbox_unbuffered!") {
            params.push(QueryParam::Bbox);
            numvars += 4;
        }
        sql = sql.replace("!bbox!", &bbox_expr);
        sql = sql.replace("!bbox_unbuffered!", &bbox_expr_unbuffered);
        // replace e.g. !zoom! with $5
        for (var, par, cast) in [
            ("!zoom!", QueryParam::Zoom, ""),
            ("!x!", QueryParam::X, ""),
            ("!y!", QueryParam::Y, ""),
            ("!pixel_width!", QueryParam::PixelWidth, "FLOAT8"),
            (
                "!scale_denominator!",
                QueryParam::ScaleDenominator,
                "FLOAT8",
            ),
        ] {
            if sql.contains(var) {
                params.push(par);
                numvars += 1;
                if !cast.is_empty() {
                    sql = sql.replace(var, &format!("${numvars}::{cast}"));
                } else {
                    sql = sql.replace(var, &format!("${numvars}"));
                }
            }
        }
        // Search and replace field query vars
        let mut unique_fields = BTreeSet::new();
        let re = Regex::new(r"!(\w+)!").expect("regex");
        for (_, [field]) in re.captures_iter(&sql).map(|c| c.extract()) {
            unique_fields.insert(field.to_string());
        }
        for field in unique_fields {
            params.push(QueryParam::QueryField(field.to_string()));
            numvars += 1;
            sql = sql.replace(&format!("!{field}!"), &format!("${numvars}"));
        }
        SqlQuery { sql, params }
    }

    pub fn param_types(&self) -> Vec<PgTypeInfo> {
        self.params
            .iter()
            .flat_map(|param| match param {
                QueryParam::Bbox => vec![
                    PgTypeInfo::with_name("FLOAT8"),
                    PgTypeInfo::with_name("FLOAT8"),
                    PgTypeInfo::with_name("FLOAT8"),
                    PgTypeInfo::with_name("FLOAT8"),
                ],
                QueryParam::Zoom | QueryParam::X | QueryParam::Y => {
                    vec![PgTypeInfo::with_name("INT4")]
                }
                QueryParam::PixelWidth | QueryParam::ScaleDenominator => {
                    vec![PgTypeInfo::with_name("FLOAT8")]
                }
                QueryParam::QueryField(_) => vec![PgTypeInfo::with_name("VARCHAR")],
            })
            .collect()
    }
}

/// Build geometry selection expression for feature query.
fn build_geom_expr(layer: &VectorLayerCfg, geom_name: &str, tile_srid: i32, zoom: u8) -> String {
    let layer_srid = layer.srid.unwrap_or(0);
    let mut geom_expr = String::from(geom_name as &str);

    // Convert special geometry types like curves
    match layer
        .geometry_type
        .as_ref()
        .unwrap_or(&"GEOMETRY".to_string()) as &str
    {
        "CURVEPOLYGON" | "COMPOUNDCURVE" => {
            geom_expr = format!("ST_CurveToLine({geom_expr})");
        }
        _ => {}
    };

    // Transform geometry to grid SRID
    if layer_srid <= 0 {
        warn!(
            "Layer '{}': Undefined SRS of geometry '{geom_name}' - assuming SRID {tile_srid}",
            layer.name
        );
        // TODO: Get SRS from geometry_columns instead
        geom_expr = format!("ST_SetSRID({geom_expr},{tile_srid})")
    } else if layer_srid != tile_srid {
        if layer.no_transform {
            geom_expr = format!("ST_SetSRID({geom_expr},{tile_srid})");
        } else {
            info!(
                "Layer '{}' z{zoom}: Reprojecting geometry '{geom_name}' from SRID {layer_srid} to {tile_srid}",
                layer.name
            );
            geom_expr = format!("ST_Transform({geom_expr},{tile_srid})");
        }
    }

    // Simplify
    if layer.simplify(zoom) {
        geom_expr = match layer
            .geometry_type
            .as_ref()
            .unwrap_or(&"GEOMETRY".to_string()) as &str
        {
            "LINESTRING" | "MULTILINESTRING" | "COMPOUNDCURVE" => format!(
                "ST_Multi(ST_SimplifyPreserveTopology({},{}))",
                geom_expr,
                layer.tolerance(zoom)
            ),
            "POLYGON" | "MULTIPOLYGON" | "CURVEPOLYGON" => {
                if layer.make_valid {
                    format!(
                    "ST_CollectionExtract(ST_Multi(ST_MakeValid(ST_SnapToGrid({geom_expr}, {}))),3)::geometry(MULTIPOLYGON,{layer_srid})",
                    layer.tolerance(zoom)
                )
                } else {
                    let empty_geom = format!("ST_GeomFromText('MULTIPOLYGON EMPTY',{layer_srid})");
                    format!(
                        "COALESCE(ST_SnapToGrid({geom_expr}, {}),{empty_geom})::geometry(MULTIPOLYGON,{layer_srid})",
                        layer.tolerance(zoom),
                    )
                }
            }
            _ => geom_expr, // No simplification for points or unknown types
        };
    }

    let tile_size = layer.tile_size;
    let buffer = layer.buffer_size.unwrap_or(0);
    let clip_geom = layer.buffer_size.is_some();

    geom_expr = format!(
        "ST_AsMvtGeom({geom_expr}, !bbox_unbuffered!, {tile_size}, {buffer}, {clip_geom}) AS {geom_name}"
    );

    geom_expr
}

/// Build PostGIS 2 compatible geometry selection expression for feature query.
fn build_geom_expr_postgis2(
    layer: &VectorLayerCfg,
    geom_name: &str,
    tile_srid: i32,
    zoom: u8,
) -> String {
    let layer_srid = layer.srid.unwrap_or(0);
    let mut geom_expr = String::from(geom_name as &str);

    // Convert special geometry types like curves
    match layer
        .geometry_type
        .as_ref()
        .unwrap_or(&"GEOMETRY".to_string()) as &str
    {
        "CURVEPOLYGON" | "COMPOUNDCURVE" => {
            geom_expr = format!("ST_CurveToLine({geom_expr})");
        }
        _ => {}
    };

    // Clipping
    if layer.buffer_size.is_some() {
        let valid_geom = if layer.make_valid {
            format!("ST_MakeValid({geom_expr})")
        } else {
            geom_expr.clone()
        };
        match layer
            .geometry_type
            .as_ref()
            .unwrap_or(&"GEOMETRY".to_string()) as &str
        {
            "POLYGON" | "MULTIPOLYGON" | "CURVEPOLYGON" => {
                geom_expr = format!("ST_Buffer(ST_Intersection({valid_geom},!bbox!), 0.0)");
            }
            "POINT" if layer_srid == tile_srid => {
                // ST_Intersection not necessary - bbox query in WHERE clause is sufficient
            }
            _ => {
                geom_expr = format!("ST_Intersection({valid_geom},!bbox!)");
            } //Buffer is added to !bbox! when replaced
        };
    }

    // convert LINESTRING and POLYGON to multi geometries (and fix potential (empty) single types)
    match layer
        .geometry_type
        .as_ref()
        .unwrap_or(&"GEOMETRY".to_string()) as &str
    {
        "MULTIPOINT" | "LINESTRING" | "MULTILINESTRING" | "COMPOUNDCURVE" | "POLYGON"
        | "MULTIPOLYGON" | "CURVEPOLYGON" => {
            geom_expr = format!("ST_Multi({geom_expr})");
        }
        _ => {}
    }

    // Simplify
    if layer.simplify(zoom) {
        geom_expr = match layer
            .geometry_type
            .as_ref()
            .unwrap_or(&"GEOMETRY".to_string()) as &str
        {
            "LINESTRING" | "MULTILINESTRING" | "COMPOUNDCURVE" => format!(
                "ST_Multi(ST_SimplifyPreserveTopology({},{}))",
                geom_expr,
                layer.tolerance(zoom)
            ),
            "POLYGON" | "MULTIPOLYGON" | "CURVEPOLYGON" => {
                if layer.make_valid {
                    format!(
                    "ST_CollectionExtract(ST_Multi(ST_MakeValid(ST_SnapToGrid({geom_expr}, {}))),3)::geometry(MULTIPOLYGON,{layer_srid})",
                    layer.tolerance(zoom)
                )
                } else {
                    let empty_geom = format!("ST_GeomFromText('MULTIPOLYGON EMPTY',{layer_srid})");
                    format!(
                        "COALESCE(ST_SnapToGrid({geom_expr}, {}),{empty_geom})::geometry(MULTIPOLYGON,{layer_srid})",
                        layer.tolerance(zoom),
                    )
                }
            }
            _ => geom_expr, // No simplification for points or unknown types
        };
    }

    // Transform geometry to grid SRID
    if layer_srid <= 0 {
        warn!(
            "Layer '{}': Undefined SRS of geometry '{geom_name}' - assuming SRID {tile_srid}",
            layer.name
        );
        geom_expr = format!("ST_SetSRID({geom_expr},{tile_srid})")
    } else if layer_srid != tile_srid {
        if layer.no_transform {
            geom_expr = format!("ST_SetSRID({geom_expr},{tile_srid})");
        } else {
            info!(
                "Layer '{}' z{zoom}: Reprojecting geometry '{geom_name}' from SRID {layer_srid} to {tile_srid}",
                layer.name
            );
            geom_expr = format!("ST_Transform({geom_expr},{tile_srid})");
        }
    }

    if geom_expr.starts_with("ST_") || geom_expr.starts_with("COALESCE") {
        geom_expr = format!("{geom_expr} AS {geom_name}");
    }

    geom_expr
}

/// Build select list expressions for feature query.
fn build_select_list(geom_expr: String, data_columns: &[FieldInfo]) -> String {
    let cols: Vec<String> = data_columns
        .iter()
        .filter_map(|col| {
            match &col.info {
                FieldTypeInfo::Property(pg_type) => {
                    // Wrap column names in double quotes to guarantee validity.
                    if pg_type.name() == "NUMERIC" {
                        // Cast to supported type
                        Some(format!(r#""{}"::FLOAT8"#, col.name))
                    } else {
                        Some(format!(r#""{}""#, col.name))
                    }
                }
                FieldTypeInfo::Geometry => Some(geom_expr.clone()),
                _ => None,
            }
        })
        .collect();
    cols.join(",")
}

/// Build !bbox! replacement expression for feature query.
fn build_bbox_expr(layer: &VectorLayerCfg, tile_srid: i32, buffer_size: Option<u32>) -> String {
    let layer_srid = layer.srid.unwrap_or(tile_srid); // we assume tile srid as default
    let env_srid = if layer_srid <= 0 || layer.no_transform {
        layer_srid
    } else {
        tile_srid
    };
    let mut expr = format!("ST_MakeEnvelope($1,$2,$3,$4,{env_srid})");
    if let Some(pixels) = buffer_size {
        if pixels != 0 {
            expr = format!("ST_MakeEnvelope($1-{p}*!pixel_width!,$2-{p}*!pixel_width!,$3+{p}*!pixel_width!,$4+{p}*!pixel_width!,{srid})",
                srid=env_srid, p=pixels);
        }
    }
    if layer_srid > 0 && layer_srid != env_srid && !layer.no_transform {
        // Note: In t-rex max_segment_length is ($3-$1)/512 instead of pixel_width
        expr = format!("ST_Transform(ST_Segmentize({expr}, !pixel_width!), {layer_srid})",);
    }
    // Clip bbox to maximal extent of SRID
    if layer.shift_longitude {
        expr = format!("ST_Shift_Longitude({expr})");
    }
    expr
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::VectorLayerQueryCfg;

    fn layer_cfg() -> (VectorLayerCfg, Vec<FieldInfo>) {
        let layer = VectorLayerCfg {
            name: "osm_place_point".to_string(),
            geometry_field: Some("geometry".to_string()),
            geometry_type: None,
            srid: Some(3857),
            no_transform: false,
            fid_field: None,
            table_name: Some("osm_place_point".to_string()),
            query_limit: None,
            queries: Vec::new(),
            minzoom: None,
            maxzoom: None,
            tile_size: 256,
            simplify: false,
            tolerance: "!pixel_width!/2".to_string(),
            buffer_size: None,
            make_valid: false,
            shift_longitude: false,
        };
        let fields = vec![FieldInfo {
            name: "geometry".to_string(),
            info: FieldTypeInfo::Geometry,
        }];
        (layer, fields)
    }

    #[test]
    fn test_basic_query() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = false;
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)"
        );
        layer.srid = None;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_SetSRID(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
    }

    #[test]
    fn test_basic_query_pg2() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = true;
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)"
        );
        layer.srid = None;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_SetSRID(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
    }

    #[test]
    fn test_reprojection() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = false;
        layer.srid = Some(2056);
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_Transform(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 2056)");
        layer.no_transform = true;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_SetSRID(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,2056)");
        layer.no_transform = false;
        layer.srid = Some(4326);
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_AsMvtGeom(ST_Transform(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 4326)"
        );
        layer.shift_longitude = true;
        assert_eq!(
        SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_AsMvtGeom(ST_Transform(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_Shift_Longitude(ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 4326))"
        );
        layer.shift_longitude = false;
        layer.srid = Some(-1);
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_SetSRID(geometry,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,-1)");
    }

    #[test]
    fn test_reprojection_pg2() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = true;
        layer.srid = Some(2056);
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Transform(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 2056)");
        layer.no_transform = true;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_SetSRID(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,2056)");
        layer.no_transform = false;
        layer.srid = Some(4326);
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_Transform(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 4326)"
        );
        layer.shift_longitude = true;
        assert_eq!(
        SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_Transform(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_Shift_Longitude(ST_Transform(ST_Segmentize(ST_MakeEnvelope($1,$2,$3,$4,3857), $5::FLOAT8), 4326))"
        );
        layer.shift_longitude = false;
        layer.srid = Some(-1);
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_SetSRID(geometry,3857) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,-1)");
    }

    #[test]
    fn test_simplification() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = false;
        layer.simplify = true;
        layer.geometry_type = Some("POLYGON".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(COALESCE(ST_SnapToGrid(geometry, $5::FLOAT8/2),ST_GeomFromText('MULTIPOLYGON EMPTY',3857))::geometry(MULTIPOLYGON,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.make_valid = true;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_CollectionExtract(ST_Multi(ST_MakeValid(ST_SnapToGrid(geometry, $5::FLOAT8/2))),3)::geometry(MULTIPOLYGON,3857), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.geometry_type = Some("LINESTRING".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_Multi(ST_SimplifyPreserveTopology(geometry,$5::FLOAT8/2)), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.tolerance = "0.5".to_string();
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_AsMvtGeom(ST_Multi(ST_SimplifyPreserveTopology(geometry,0.5)), ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.geometry_type = Some("POINT".to_string());
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)"
        );
    }

    #[test]
    fn test_simplification_pg2() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = true;
        layer.simplify = true;
        layer.geometry_type = Some("POLYGON".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT COALESCE(ST_SnapToGrid(ST_Multi(geometry), $5::FLOAT8/2),ST_GeomFromText('MULTIPOLYGON EMPTY',3857))::geometry(MULTIPOLYGON,3857) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.make_valid = true;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_CollectionExtract(ST_Multi(ST_MakeValid(ST_SnapToGrid(ST_Multi(geometry), $5::FLOAT8/2))),3)::geometry(MULTIPOLYGON,3857) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.geometry_type = Some("LINESTRING".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Multi(ST_SimplifyPreserveTopology(ST_Multi(geometry),$5::FLOAT8/2)) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.tolerance = "0.5".to_string();
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Multi(ST_SimplifyPreserveTopology(ST_Multi(geometry),0.5)) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
        layer.geometry_type = Some("POINT".to_string());
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)"
        );
    }

    #[test]
    fn test_clipping_pg2() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = true;
        layer.buffer_size = Some(10);
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Intersection(geometry,ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)");
        layer.make_valid = true;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Intersection(ST_MakeValid(geometry),ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)");
        layer.geometry_type = Some("POLYGON".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT ST_Multi(ST_Buffer(ST_Intersection(ST_MakeValid(geometry),ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)), 0.0)) AS geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)");
        layer.geometry_type = Some("POINT".to_string());
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
               "SELECT geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1-10*$5::FLOAT8,$2-10*$5::FLOAT8,$3+10*$5::FLOAT8,$4+10*$5::FLOAT8,3857)");
        layer.buffer_size = Some(0);
        assert_eq!(
            SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, None, postgis2).sql,
            "SELECT geometry FROM osm_place_point WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)"
        );
    }

    #[test]
    fn test_user_queries() {
        let (mut layer, fields) = layer_cfg();
        layer.queries = vec![VectorLayerQueryCfg {
            minzoom: Some(0),
            maxzoom: Some(22),
            simplify: None,
            tolerance: None,
            sql: Some(String::from("SELECT geometry FROM osm_place_point")),
        }];
        let postgis2 = false;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, layer.queries[0].sql.as_ref(), postgis2)
                   .sql,
               "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM (SELECT geometry FROM osm_place_point) AS _q WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");

        layer.queries = vec![VectorLayerQueryCfg {
            minzoom: Some(0),
            maxzoom: Some(22),
            simplify: None,
            tolerance: None,
            sql: Some(String::from(
                "SELECT * FROM osm_place_point WHERE name='Bern'",
            )),
        }];
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, layer.queries[0].sql.as_ref(), postgis2)
                   .sql,
               "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM (SELECT * FROM osm_place_point WHERE name='Bern') AS _q WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
    }

    #[test]
    fn test_xyz_queries() {
        let (mut layer, fields) = layer_cfg();
        layer.queries = vec![VectorLayerQueryCfg {
            minzoom: Some(0),
            maxzoom: Some(22),
            simplify: None,
            tolerance: None,
            sql: Some(String::from(
                "SELECT geom FROM prepared_tiles WHERE x=!x! AND y=!y! and z=!zoom!",
            )),
        }];
        let postgis2 = false;
        assert_eq!(SqlQuery::build_tile_query(&layer, "geom", &fields, 3857, 10, layer.queries[0].sql.as_ref(), postgis2)
                   .sql,
               "SELECT ST_AsMvtGeom(geom, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geom FROM (SELECT geom FROM prepared_tiles WHERE x=$6 AND y=$7 and z=$5) AS _q");
    }

    #[test]
    fn test_field_query() {
        let (mut layer, fields) = layer_cfg();
        let postgis2 = false;
        layer.queries = vec![VectorLayerQueryCfg {
            minzoom: Some(0),
            maxzoom: Some(22),
            simplify: None,
            tolerance: None,
            sql: Some(String::from(
                "SELECT geometry FROM osm_place_point WHERE col1=!colval1! AND col2=!colval2!",
            )),
        }];
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, layer.queries[0].sql.as_ref(), postgis2)
                   .sql,
               "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM (SELECT geometry FROM osm_place_point WHERE col1=$5 AND col2=$6) AS _q WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");

        layer.queries = vec![VectorLayerQueryCfg {
            minzoom: Some(0),
            maxzoom: Some(22),
            simplify: None,
            tolerance: None,
            sql: Some(String::from(
                "SELECT geometry FROM osm_place_point WHERE col1=!colval! OR col2=!colval!",
            )),
        }];
        assert_eq!(SqlQuery::build_tile_query(&layer, "geometry", &fields, 3857, 10, layer.queries[0].sql.as_ref(), postgis2)
                   .sql,
               "SELECT ST_AsMvtGeom(geometry, ST_MakeEnvelope($1,$2,$3,$4,3857), 256, 0, false) AS geometry FROM (SELECT geometry FROM osm_place_point WHERE col1=$5 OR col2=$5) AS _q WHERE geometry && ST_MakeEnvelope($1,$2,$3,$4,3857)");
    }
}

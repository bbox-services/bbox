use crate::tilesource::TileSource;
use bbox_core::cli::CommonCommands;
use bbox_core::config::{from_config_root_or_exit, NamedDatasourceCfg};
use clap::{ArgMatches, FromArgMatches};
use log::info;
use serde::Deserialize;
use std::num::NonZeroU16;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct TileserverCfg {
    #[serde(rename = "grid")]
    pub grids: Vec<GridCfg>,
    #[serde(rename = "datasource")]
    pub datasources: Vec<NamedDatasourceCfg>,
    #[serde(rename = "tileset")]
    pub tilesets: Vec<TileSetCfg>,
    #[serde(rename = "tilecache")]
    pub tilecaches: Vec<TileCacheProviderCfg>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileSetCfg {
    pub name: String,
    /// Tile format (Default: Raster)
    // pub format: Option<TileFormatCfg>,
    /// List of available tile matrix set identifiers (Default: WebMercatorQuad)
    pub tms: Option<String>,
    /// Source parameters
    #[serde(flatten)]
    pub params: SourceParamCfg,
    /// Tile cache name (Default: no cache)
    pub cache: Option<String>,
    pub cache_limits: Option<CacheLimitCfg>,
}

/// Custom grid definition
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GridCfg {
    /// JSON file path
    pub json: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub enum SourceParamCfg {
    #[serde(rename = "wms_proxy")]
    WmsHttp(WmsHttpSourceParamsCfg),
    #[serde(rename = "wms_project")]
    WmsFcgi(WmsFcgiSourceParamsCfg),
    #[serde(rename = "postgis")]
    Postgis(PostgisSourceParamsCfg),
    #[serde(rename = "mbtiles")]
    Mbtiles(MbtilesSourceParamsCfg),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsHttpSourceParamsCfg {
    /// name of WmsHttpSourceProviderCfg
    pub source: String,
    pub layers: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WmsFcgiSourceParamsCfg {
    pub project: String,
    pub suffix: String,
    pub layers: String,
    /// Additional WMS params like transparent=true
    pub params: Option<String>,
    /// Width and height of tile. Defaults to grid tile size (usually 256x256)
    // TODO: per layer for MVT, investigate for OGC Tiles
    pub tile_size: Option<NonZeroU16>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct MbtilesSourceParamsCfg {
    pub path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PostgisSourceParamsCfg {
    /// Name of tileserver.source config (Default: first with matching type)
    // maybe we should allow direct DS URLs?
    // t-rex has datasource on layer level!
    pub datasource: Option<String>,
    pub extent: Option<ExtentCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub center: Option<(f64, f64)>,
    pub start_zoom: Option<u8>,
    pub attribution: Option<String>,
    #[serde(rename = "layer")]
    pub layers: Vec<VectorLayerCfg>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtentCfg {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VectorLayerCfg {
    pub name: String,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    /// Spatial reference system (PostGIS SRID)
    pub srid: Option<i32>,
    /// Assume geometry is in grid SRS
    #[serde(default)]
    pub no_transform: bool,
    /// Name of feature ID field
    pub fid_field: Option<String>,
    // Input for derived queries
    pub table_name: Option<String>,
    pub query_limit: Option<u32>,
    // Custom queries
    #[serde(default)]
    pub query: Vec<VectorLayerQueryCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    /// Width and height of the tile (Default: 4096. Grid default size is 256)
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    /// Simplify geometry (lines and polygons)
    #[serde(default)]
    pub simplify: bool,
    /// Simplification tolerance (default to !pixel_width!/2)
    #[serde(default = "default_tolerance")]
    pub tolerance: String,
    /// Tile buffer size in pixels (None: no clipping)
    pub buffer_size: Option<u32>,
    /// Fix invalid geometries before clipping (lines and polygons)
    #[serde(default)]
    pub make_valid: bool,
    /// Apply ST_Shift_Longitude to (transformed) bbox
    #[serde(default)]
    pub shift_longitude: bool,
}

fn default_tile_size() -> u32 {
    4096
}

const DEFAULT_TOLERANCE: &str = "!pixel_width!/2";

fn default_tolerance() -> String {
    DEFAULT_TOLERANCE.to_string()
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct VectorLayerQueryCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    /// Simplify geometry (override layer default setting)
    pub simplify: Option<bool>,
    /// Simplification tolerance (override layer default setting)
    pub tolerance: Option<String>,
    pub sql: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct CacheLimitCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileCacheProviderCfg {
    pub name: String,
    // pub layout: CacheLayout,
    #[serde(flatten)]
    pub cache: TileCacheCfg,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum TileCacheCfg {
    Files(FileCacheCfg),
    S3(S3CacheCfg),
    // MbTiles(MbTilesCache),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct FileCacheCfg {
    pub base_dir: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct S3CacheCfg {
    pub path: String,
    // pub s3_endpoint_url: Option<String>,
    // pub aws_access_key_id: Option<String>,
    // pub aws_secret_access_key: Option<String>,
}

impl TileserverCfg {
    pub fn from_config(cli: &ArgMatches) -> Self {
        let mut cfg: TileserverCfg = from_config_root_or_exit();

        // Get config from CLI
        if let Ok(CommonCommands::Serve(args)) = CommonCommands::from_arg_matches(cli) {
            if let Some(file_or_url) = args.file_or_url {
                if let Some(source_cfg) = TileSource::config_from_cli_arg(&file_or_url) {
                    let name = if let Some(name) = Path::new(&file_or_url).file_stem() {
                        name.to_string_lossy().to_string()
                    } else {
                        file_or_url.to_string()
                    };
                    info!("Adding tileset `{name}`");
                    let ts = TileSetCfg {
                        name,
                        tms: None,
                        params: source_cfg,
                        cache: None,
                        cache_limits: None,
                    };
                    cfg.tilesets.push(ts);
                }
            }
        }
        cfg
    }
}

#[allow(dead_code)]
static WORLD_EXTENT: ExtentCfg = ExtentCfg {
    minx: -180.0,
    miny: -90.0,
    maxx: 180.0,
    maxy: 90.0,
};

#[allow(dead_code)]
impl PostgisSourceParamsCfg {
    pub fn minzoom(&self) -> u8 {
        self.minzoom
            .unwrap_or(self.layers.iter().map(|l| l.minzoom()).min().unwrap_or(0))
    }
    pub fn maxzoom(&self) -> u8 {
        self.maxzoom.unwrap_or(
            self.layers
                .iter()
                .map(|l| l.maxzoom(22))
                .max()
                .unwrap_or(22),
        )
    }
    pub fn attribution(&self) -> String {
        self.attribution.clone().unwrap_or("".to_string())
    }
    pub fn get_extent(&self) -> &ExtentCfg {
        self.extent.as_ref().unwrap_or(&WORLD_EXTENT)
    }
    pub fn get_center(&self) -> (f64, f64) {
        if self.center.is_none() {
            let ext = self.get_extent();
            (
                ext.maxx - (ext.maxx - ext.minx) / 2.0,
                ext.maxy - (ext.maxy - ext.miny) / 2.0,
            )
        } else {
            self.center.unwrap()
        }
    }
    pub fn get_start_zoom(&self) -> u8 {
        self.start_zoom.unwrap_or(2)
    }
}

impl VectorLayerCfg {
    pub fn minzoom(&self) -> u8 {
        self.minzoom
            .unwrap_or(self.query.iter().map(|q| q.minzoom).min().unwrap_or(0))
    }
    pub fn maxzoom(&self, default: u8) -> u8 {
        self.maxzoom.unwrap_or(
            self.query
                .iter()
                .map(|q| q.maxzoom.unwrap_or(default))
                .max()
                .unwrap_or(default),
        )
    }
    /// Collect min zoom levels with configuration
    pub fn zoom_steps(&self) -> Vec<u8> {
        let mut zoom_steps = self
            .query
            .iter()
            .filter(|q| q.sql.is_some())
            .map(|q| q.minzoom)
            .collect::<Vec<_>>();
        zoom_steps.sort();
        if zoom_steps.is_empty() {
            zoom_steps.push(self.minzoom());
        } else if self.minzoom() < zoom_steps[0] {
            zoom_steps.insert(0, self.minzoom());
        }
        zoom_steps
    }
    /// Query config for zoom level
    fn query_cfg<F>(&self, level: u8, check: F) -> Option<&VectorLayerQueryCfg>
    where
        F: Fn(&VectorLayerQueryCfg) -> bool,
    {
        let mut queries = self
            .query
            .iter()
            .map(|q| (q.minzoom, q.maxzoom.unwrap_or(22), q))
            .collect::<Vec<_>>();
        queries.sort_by_key(|t| t.0);
        // Start at highest zoom level and find first match
        let query = queries
            .iter()
            .rev()
            .find(|q| level >= q.0 && level <= q.1 && check(q.2));
        query.map(|q| q.2)
    }
    /// SQL query for zoom level
    pub fn query(&self, level: u8) -> Option<&String> {
        let query_cfg = self.query_cfg(level, |q| q.sql.is_some());
        query_cfg.and_then(|q| q.sql.as_ref())
    }
    /// simplify config for zoom level
    pub fn simplify(&self, level: u8) -> bool {
        let query_cfg = self.query_cfg(level, |q| q.simplify.is_some());
        query_cfg.and_then(|q| q.simplify).unwrap_or(self.simplify)
    }
    /// tolerance config for zoom level
    pub fn tolerance(&self, level: u8) -> &String {
        let query_cfg = self.query_cfg(level, |q| q.tolerance.is_some());
        query_cfg
            .and_then(|q| q.tolerance.as_ref())
            .unwrap_or(&self.tolerance)
    }
    // Layer properties needed e.g. for metadata.json
    // pub fn metadata(&self) -> HashMap<&str, String> {
    //     //TODO: return Zoom-Level Array
    //     let mut metadata: HashMap<&str, String> = HashMap::new();
    //     metadata.insert("id", self.name.clone());
    //     metadata.insert("name", self.name.clone());
    //     metadata.insert("description", "".to_string());
    //     metadata.insert("buffer-size", self.buffer_size.unwrap_or(0).to_string());
    //     metadata.insert("minzoom", self.minzoom().to_string());
    //     metadata.insert("maxzoom", self.maxzoom(22).to_string());
    //     //metadata.insert("srs", "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over".to_string());
    //     metadata
    // }
}

// Mapproxy Yaml:

// services:
//   demo:

//   wmts:
//     restful: true
//     featureinfo_formats:
//       - mimetype: application/gml+xml; version=3.1
//         suffix: gml
//       - mimetype: text/html
//         suffix: html
//     md:
//       title: "Gedati relativi al territorio del Canton Ticino"
//       abstract: Geodati di base relativi al territorio della Repubblica e Canton Ticino esposti tramite geoservizi WMTS. L 'organizzazione dei geodati di base è secondo le geocategorie definite nella norma eCH0166. I geodati di base vengono offerti secondo i servizi, di rappresentazione (WMTS), definiti dal Regolamento della legge cantonale sulla geoinformazione.
//       online_resource: https://dev.geo.ti.ch/wmts/1.0.0/WMTSCapabilities.xml
//       contact:
//         person: Ufficio della geomatica
//         position: Point of contact
//         organization: Repubblica e Cantone Ticino
//         address: Via Franco Zorzi 13
//         city: Bellinzona
//         postcode: 6500
//         state: Ticino
//         country: Switzerland
//         phone: +41(91)814 26 15
//         fax: +41(91)814 25 29
//         email: ccgeo@ti.ch
//       access_constraints: Richiesta formale a ccgeo@ti.ch
//       fees: 'None'
//       keyword_list:
//        - vocabulary: GEMET
//        - keywords:   [MU]
//        - keywords:   [Geodati di base, Dati territoriali]
//   wms:
//     srs: ['EPSG:4326','EPSG:3857', 'EPSG:21781', 'EPSG:2056']
//     # force the layer extents (BBOX) to be displayed in this SRS
//     # bbox_srs: ['EPSG:4326','EPSG:3857', 'EPSG:21781']
//     # attribution:
//       # text: "© Amministrazione cantonale"
//     versions: ['1.0.0', '1.1.0', '1.1.1', '1.3.0']
//     #versions: ['1.3.0']
//     bbox_srs:
//       - 'EPSG:4326'
//       - 'EPSG:3857'
//       - 'EPSG:2056'
//       - srs:'EPSG:2056'
//         bbox [2420000.00,1030000.00,2900000.00,1350000.00]
//     md:
//       title: "Geoservizi dei dati relativi al territorio del Canton Ticino"
//       abstract: Geoservizi (WMS/WFS) espongono i geodati di base relativi al territorio della Repubblica e Canton Ticino. L'organizzazione dei geodati di base è secondo le geocategorie definite nella norma eCH0166. I geodati di base vengono offerti secondo i servizi, di rappresentazione (WMS) o di telecaricamento (WFS), definiti dal Regolamento della legge cantonale sulla geoinformazione.
//       online_resource: https://dev.geo.ti.ch/service?
//       contact:
//         person: Ufficio della geomatica
//         position: Point of contact
//         organization: Repubblica e Cantone Ticino
//         address: Via Franco Zorzi 13
//         city: Bellinzona
//         postcode: 6500
//         state: Ticino
//         country: Switzerland
//         phone: +41(91)814 26 15
//         fax: +41(91)814 25 29
//         email: ccgeo@ti.ch
//       access_constraints: Richeista formale a ccgeo@ti.ch
//       fees: 'None'
//       keyword_list:
//        - vocabulary: GEMET
//        - keywords:   [MU]
//        - keywords:   [Geodati di base, Dati territoriali]

// base: [layers.yaml,caches.yaml,sources.yaml]

// grids:
//     webmercator:
//         base: GLOBAL_WEBMERCATOR

//     ch_grid:
//         srs: 'EPSG:21781'
//         bbox: [420000.00,30000.00,920000.00,350000.00]
//         origin: nw
//         tile_size : [ 256 , 256 ]
//         # resolutions created from scales with
//         res: [4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.125,0.1,0.0625]

//     ch95_grid:
//         srs: 'EPSG:2056'
//         bbox: [2420000.00,1030000.00,2920000.00,1350000.00]
//         origin: nw
//         tile_size : [ 256 , 256 ]
//         # resolutions created from scales with
//         res: [4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.125,0.1,0.0625]

// # -- caches.yaml

// caches:
//   51_1_color_cache:
//     grids:
//     - ch95_grid
//     sources:
//     - 51-1_color
//     bulk_meta_tiles: true
//     cache:
//       #type: sqlite
//       #directory: /mapproxy/cache_data/51-1_color
//       type: file
//       directory: /home/marco/tmp/tiles
//       #directory: /home/marco/officepc/tile_caches/ti_51-1_color
//       #settings for s3
//       #type: s3
//       #bucket_name: tiles
//       #endpoint_url: http://officepc:9000
//       #directory: /
//       directory_layout: tms

// # -- sources.yaml

// sources:
//   51-1_color:
//     type: wms
//     wms_opts:
//       legendgraphic: false
//       featureinfo: true
//     req:
//       url: http://localhost/cgi-bin/qgis_mapserv.fcgi?MAP=/opt/qgis_server_data/ch_051_1_version1_7_mn95.qgz
//       layers: ch.ti.051_1.piano_registro_fondiario_colori
//       transparent: true
//     supported_srs:
//     - CRS:84
//     - EPSG:3857
//     - EPSG:21781
//     - EPSG:2056
//     coverage:
//       bbox:
//       - 2670330.0
//       - 1073180.0
//       - 2736990.0
//       - 1167820.0
//       srs: EPSG:2056

// # -- seed.yaml

// seeds:
//     seed_update_mu:
//         # productive configuration
//         caches: [51_1_color_cache]
//         #caches: [51_1_bn_cache,51_1_color_cache,51_1_bn_crdpp_cache]
//         #caches: [ac002_1_3_cache]
//         grids: [ch95_grid]
//         coverages: [mu_update]
//         refresh_before:
//             mtime: coverage_TI.geojson
//         levels:
//             to: 26

// <mapcache>
//   <metadata>
//     <title>WMTS / Amt für Geoinformation Kanton Solothurn</title>
//     <abstract>None</abstract>
//     <!-- <url>SERVICE_URL</url> -->
//   </metadata>

//   <grid name="2056">
//     <metadata>
//       <title>CH1903+ / LV95</title>
//     </metadata>
//     <origin>top-left</origin>
//     <srs>EPSG:2056</srs>
//     <units>m</units>
//     <extent>2420000,1030000,2900000,1350000</extent>
//     <!--eCH-0056 v2 ? / bisher -->
//     <!--<resolutions>4000,3750,3500,3250,3000,2750,2500,2250,2000,1750,1500,1250,1000,750,650,500,250,100,50,20,10,5,2.5,2,1.5,1,0.5,0.25,0.1</resolutions>-->
//     <!--eCH-0056 v3-->
//     <!--Resolution 0.05 removed intentionally from the following list-->
//     <resolutions>4000,2000,1000,500,250,100,50,20,10,5,2.5,1,0.5,0.25,0.1</resolutions>
//     <size>256 256</size>
//   </grid>

//   <cache name="sqlite" type="sqlite3">
//     <dbfile>/tiles/{tileset}-{z}-{grid}.db</dbfile>
//     <detect_blank/>
//   </cache>

//   <format name="myjpeg" type ="JPEG">
//     <quality>80</quality>
//     <photometric>YCBCR</photometric>   <!-- RGB | YCBCR -->
//   </format>

//   <source name="ch.so.agi.hintergrundkarte_ortho" type="wms">
//     <getmap>
//       <params>
//         <FORMAT>image/jpeg</FORMAT>
//         <LAYERS>ch.so.agi.hintergrundkarte_ortho</LAYERS>
//         <TRANSPARENT>true</TRANSPARENT>
//       </params>
//     </getmap>
//     <http>
//       <url>SOURCE_URL</url>
//       <connection_timeout>60</connection_timeout>
//     </http>
//   </source>

//   <tileset name="ch.so.agi.hintergrundkarte_sw">
//     <source>ch.so.agi.hintergrundkarte_sw</source>
//     <cache>sqlite</cache>
//     <grid restricted_extent="2570000,1208000,2667000,1268000">2056</grid>
//     <format>PNG</format>
//     <metatile>8 8</metatile>
//     <metabuffer>20</metabuffer>
//     <expires>28800</expires>
//   </tileset>

//   <default_format>JPEG</default_format>
//   <service type="wms" enabled="true">
//     <full_wms>assemble</full_wms>
//     <resample_mode>bilinear</resample_mode>
//     <format allow_client_override="true">JPEG</format>
//     <maxsize>4096</maxsize>
//   </service>
//   <service type="wmts" enabled="true"/>
//   <service type="tms" enabled="false"/>
//   <service type="kml" enabled="false"/>
//   <service type="gmaps" enabled="false"/>
//   <service type="ve" enabled="false"/>
//   <service type="mapguide" enabled="false"/>
//   <service type="demo" enabled="DEMO_SERVICE_ENABLED"/>
//   <errors>report</errors>
//   <locker type="disk">
//     <directory>/tmp</directory>
//     <timeout>300</timeout>
//   </locker>
// </mapcache>

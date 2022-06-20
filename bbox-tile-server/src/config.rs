use bbox_common::config::config_error_exit;
use serde::Deserialize;
use tile_grid::{Extent, Grid, Origin, Unit};

// from t-rex core gridcfg.rs

#[derive(Deserialize, Clone, Debug)]
pub struct ExtentCfg {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

impl From<&ExtentCfg> for Extent {
    fn from(cfg: &ExtentCfg) -> Extent {
        Extent {
            minx: cfg.minx,
            miny: cfg.miny,
            maxx: cfg.maxx,
            maxy: cfg.maxy,
        }
    }
}

pub trait FromGridCfg {
    fn from_config(grid_cfg: &GridCfg) -> Result<Self, String>
    where
        Self: Sized;
}

impl FromGridCfg for Grid {
    fn from_config(grid_cfg: &GridCfg) -> Result<Self, String> {
        if let Some(ref gridname) = grid_cfg.predefined {
            match gridname.as_str() {
                "wgs84" => Ok(Grid::wgs84()),
                "web_mercator" => Ok(Grid::web_mercator()),
                _ => Err(format!("Unkown grid '{}'", gridname)),
            }
        } else if let Some(ref usergrid) = grid_cfg.user {
            let units = match &usergrid.units.to_lowercase() as &str {
                "m" => Ok(Unit::Meters),
                "dd" => Ok(Unit::Degrees),
                "ft" => Ok(Unit::Feet),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.units)),
            };
            let origin = match &usergrid.origin as &str {
                "TopLeft" => Ok(Origin::TopLeft),
                "BottomLeft" => Ok(Origin::BottomLeft),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.origin)),
            };
            let grid = Grid::new(
                usergrid.width,
                usergrid.height,
                Extent::from(&usergrid.extent),
                usergrid.srid,
                units?,
                usergrid.resolutions.clone(),
                origin?,
            );
            Ok(grid)
        } else {
            Err("Invalid grid definition".to_string())
        }
    }
}

// from t-rex core config.rs

#[derive(Deserialize, Clone, Debug)]
pub struct GridCfg {
    pub predefined: Option<String>,
    pub user: Option<UserGridCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserGridCfg {
    /// The width and height of an individual tile, in pixels.
    pub width: u16,
    pub height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: ExtentCfg,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units (m: meters, dd: decimal degrees, ft: feet)
    pub units: String,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    #[serde(default)]
    pub resolutions: Vec<f64>,
    /// Grid origin
    pub origin: String,
}

impl GridCfg {
    pub fn from_config() -> Option<Self> {
        let config = bbox_common::config::app_config();
        config
            .find_value("grid")
            .map(|_| {
                config
                    .extract_inner("grid")
                    .map_err(|err| config_error_exit(err))
                    .unwrap()
            })
            .ok()
    }
}

// WMS backend

#[derive(Deserialize)]
pub struct BackendWmsCfg {
    pub baseurl: String,
    pub layers: String,
    pub format: String,
}

impl BackendWmsCfg {
    pub fn from_config() -> Option<Self> {
        let config = bbox_common::config::app_config();
        config
            .find_value("tile.wms")
            .map(|_| {
                config
                    .extract_inner("tile.wms")
                    .map_err(|err| config_error_exit(err))
                    .unwrap()
            })
            .ok()
    }
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

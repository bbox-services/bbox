use bbox_common::config::from_config_or_exit;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileserverCfg {
    #[serde(default)]
    pub tileset: Vec<TileSetCfg>,
    #[serde(default)]
    pub grid: Vec<GridCfg>,
    #[serde(default)]
    pub source: Vec<TileSourceCfg>,
    #[serde(default)]
    pub cache: Vec<TileCacheProviderCfg>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileSourceCfg {
    pub name: String,
    #[serde(flatten)]
    pub config: TileSourceProviderCfg,
}

#[derive(Deserialize, Clone, Debug)]
pub enum TileSourceProviderCfg {
    WmsFcgi,
    #[serde(rename = "wms_proxy")]
    WmsHttp(WmsHttpSourceProviderCfg),
    // GdalData(GdalSource),
    // RasterData(GeorasterSource),
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
}

/// Custom grid definition
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GridCfg {
    /// JSON file path
    pub json: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WmsHttpSourceProviderCfg {
    pub baseurl: String,
    pub format: String,
}

#[derive(Deserialize, Clone, Debug)]
pub enum SourceParamCfg {
    #[serde(rename = "wms_proxy")]
    WmsHttp(WmsHttpSourceParamsCfg),
    #[serde(rename = "wms_project")]
    WmsFcgi(WmsFcgiSourceParamsCfg),
}

#[derive(Deserialize, Clone, Debug)]
pub struct WmsHttpSourceParamsCfg {
    /// name of WmsHttpSourceProviderCfg
    pub source: String,
    pub layers: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WmsFcgiSourceParamsCfg {
    pub project: String,
    pub suffix: String,
    pub layers: String,
    /// Additional WMS params like transparent=true
    pub params: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TileCacheProviderCfg {
    pub name: String,
    // pub layout: CacheLayout,
    #[serde(flatten)]
    pub cache: TileCacheCfg,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TileCacheCfg {
    Files(FileCacheCfg),
    S3(S3CacheCfg),
    // MbTiles(MbTilesCache),
}

#[derive(Deserialize, Clone, Debug)]
pub struct FileCacheCfg {
    pub base_dir: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
pub struct S3CacheCfg {
    pub path: String,
    // pub s3_endpoint_url: Option<String>,
    // pub aws_access_key_id: Option<String>,
    // pub aws_secret_access_key: Option<String>,
}

impl TileserverCfg {
    pub fn from_config() -> Self {
        from_config_or_exit("tileserver")
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

use crate::inventory;
use crate::wms_capabilities as ogc;
use lazy_static::lazy_static;
use log::warn;
use serde::Serialize;

/// QWC2 map theme configuration
/// See https://github.com/qgis/qwc2-demo-app/blob/master/doc/QWC2_Documentation.md#themesConfig-json
#[derive(Debug, Serialize, Clone)]
pub struct ThemesJson {
    themes: Themes,
}

#[derive(Debug, Serialize, Clone)]
pub struct Themes {
    pub title: String,
    pub subdirs: Vec<Option<()>>,
    pub items: Vec<Theme>,
    #[serde(rename = "defaultTheme")]
    pub default_theme: String,
    #[serde(rename = "defaultScales")]
    pub default_scales: Vec<i64>,
    #[serde(rename = "defaultPrintGrid")]
    pub default_print_grid: Vec<GridInterval>,
    #[serde(rename = "backgroundLayers")]
    pub background_layers: Vec<BackgroundLayer>,
    #[serde(rename = "defaultWMSVersion")]
    pub default_wms_version: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Theme {
    pub url: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub attribution: Attribution,
    #[serde(rename = "abstract")]
    pub item_abstract: String,
    pub keywords: String,
    #[serde(rename = "onlineResource")]
    pub online_resource: String,
    pub contact: Contact,
    #[serde(rename = "availableFormats")]
    pub available_formats: Vec<String>,
    pub version: String,
    #[serde(rename = "infoFormats")]
    pub info_formats: Vec<String>,
    pub bbox: Bbox,
    #[serde(rename = "initialBbox")]
    pub initial_bbox: Bbox,
    #[serde(rename = "printResolutions")]
    pub print_resolutions: Vec<i64>,
    pub sublayers: Vec<Layer>,
    pub expanded: bool,
    #[serde(rename = "externalLayers")]
    pub external_layers: Vec<Option<()>>,
    #[serde(rename = "backgroundLayers")]
    pub background_layers: Vec<ThemeBackgroundLayer>,
    #[serde(rename = "searchProviders")]
    pub search_providers: Vec<String>,
    #[serde(rename = "additionalMouseCrs")]
    pub additional_mouse_crs: Vec<Option<()>>,
    #[serde(rename = "mapCrs")]
    pub map_crs: String,
    #[serde(rename = "drawingOrder")]
    pub drawing_order: Vec<String>,
    #[serde(rename = "legendUrl")]
    pub legend_url: String,
    #[serde(rename = "featureInfoUrl")]
    pub feature_info_url: String,
    #[serde(rename = "printUrl")]
    pub print_url: String,
    #[serde(rename = "skipEmptyFeatureAttributes")]
    pub skip_empty_feature_attributes: bool,
    #[serde(rename = "editConfig")]
    pub edit_config: Option<()>,
    pub thumbnail: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Layer {
    pub name: String,
    pub title: String,
    pub visibility: Option<bool>,
    pub queryable: Option<bool>,
    #[serde(rename = "displayField")]
    pub display_field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
    #[serde(rename = "dataUrl", skip_serializing_if = "Option::is_none")]
    pub data_url: Option<String>,
    #[serde(rename = "metadataUrl", skip_serializing_if = "Option::is_none")]
    pub metadata_url: Option<String>,
    pub opacity: Option<u8>,
    #[serde(rename = "minScale", skip_serializing_if = "Option::is_none")]
    pub min_scale: Option<i64>,
    #[serde(rename = "maxScale", skip_serializing_if = "Option::is_none")]
    pub max_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Bbox>,
    #[serde(rename = "mutuallyExclusive", skip_serializing_if = "Option::is_none")]
    pub mutually_exclusive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sublayers: Option<Vec<Layer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Attribution {
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "OnlineResource")]
    pub online_resource: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Contact {
    pub person: String,
    pub organization: String,
    pub position: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Bbox {
    pub crs: String,
    pub bounds: Vec<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GridInterval {
    pub s: i64,
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ThemeBackgroundLayer {
    pub name: String,
    #[serde(rename = "printLayer")]
    pub print_layer: String,
    pub visibility: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct BackgroundLayer {
    pub name: String,
    pub title: String,
    #[serde(rename = "type")]
    pub background_layer_type: String,
    pub source: Option<String>,
    pub thumbnail: String,
    pub attribution: Attribution,
    pub url: Option<String>,
    #[serde(rename = "tileMatrixPrefix")]
    pub tile_matrix_prefix: Option<String>,
    #[serde(rename = "tileMatrixSet")]
    pub tile_matrix_set: Option<String>,
    #[serde(rename = "originX")]
    pub origin_x: Option<f64>,
    #[serde(rename = "originY")]
    pub origin_y: Option<f64>,
    pub projection: Option<String>,
    pub resolutions: Option<Vec<f64>>,
    #[serde(rename = "tileSize")]
    pub tile_size: Option<[u16; 2]>,
}

impl ThemesJson {
    pub fn from_capabilities(
        caps: Vec<(&inventory::WmsService, ogc::WmsCapabilities, String)>,
        default_theme: Option<&str>,
    ) -> Self {
        let themes: Vec<Theme> = caps
            .iter()
            .map(|(wms, c, url)| Theme::from_capabilities(wms.id.to_string(), c, url).unwrap())
            .collect();
        let default_theme = default_theme
            .or(caps.get(0).map(|(wms, _, _)| wms.id.as_str()))
            .unwrap_or("")
            .to_string();
        ThemesJson {
            themes: Themes {
                title: "themes".to_string(),
                subdirs: vec![],
                items: themes,
                default_theme,
                default_scales: THEMES_JSON.themes.default_scales.clone(), // TODO
                default_print_grid: THEMES_JSON.themes.default_print_grid.clone(), // TODO
                background_layers: THEMES_JSON.themes.background_layers.clone(), // TODO
                default_wms_version: "1.3.0".to_string(),
            },
        }
    }
}

impl Theme {
    pub fn from_capabilities(
        id: String,
        caps: &ogc::WmsCapabilities,
        url: &String,
    ) -> Option<Self> {
        if caps.capability.layers.len() == 0 {
            return None;
        }
        if caps.capability.layers.len() > 1 {
            warn!("Ignoring root layers after first one in WMS capabilitiies")
        }
        let layers = parse_layer_tree(&caps.capability.layers);

        let root_layer_ogc = &caps.capability.layers[0];
        let _root_layer = &layers[0];
        let sublayers = layers[0].sublayers.as_ref().unwrap_or(&vec![]).clone();
        let keywords = "".to_string();
        // capabilities.Service.KeywordList.Keyword).map((entry) => {
        //     let value = (typeof entry === 'object') ? entry._ : entry;
        //     if (value !== "infoMapAccessService") {
        //         keywords.push(value);
        //     }
        // };
        // Search for first layer with valid bbox (UMN may have invalid bbox on root level)
        let bbox_reference_layer = caps
            .capability
            .layers
            .iter()
            .chain(root_layer_ogc.layers.iter())
            .find(|l| {
                if let Some(bbox) = &l.ex_geographic_bounding_box {
                    (bbox.west_bound_longitude - bbox.east_bound_longitude).abs() > 0.0
                        && (bbox.south_bound_latitude - bbox.north_bound_latitude).abs() > 0.0
                } else {
                    false
                }
            })
            .unwrap_or(root_layer_ogc);
        let bbox = bbox_reference_layer
            .ex_geographic_bounding_box
            .as_ref()
            .map(|c| Bbox {
                crs: "EPSG:4326".to_string(),
                bounds: vec![
                    c.west_bound_longitude,
                    c.south_bound_latitude,
                    c.east_bound_longitude,
                    c.north_bound_latitude,
                ],
            })
            .unwrap_or(Bbox {
                crs: "EPSG:4326".to_string(),
                bounds: vec![-180.0, -90.0, 180.0, 90.0],
            });
        let map_crs = root_layer_ogc
            .crs
            .iter()
            .find(|&entry| entry.to_uppercase() != "CRS:84")
            .map(|crs| crs.to_uppercase());
        // let first_bbox: Option<Bbox> = map_crs.as_ref().and_then(|crs| {
        //     bbox_reference_layer
        //         .bounding_box
        //         .iter()
        //         .find(|bbox| &bbox.crs.to_uppercase() == crs)
        //         .map(|bbox| Bbox {
        //             crs: bbox.crs.to_uppercase(),
        //             bounds: vec![bbox.minx, bbox.miny, bbox.maxx, bbox.maxy],
        //         })
        // });

        let theme = Theme {
            url: url.clone(),
            id,
            title: caps.service.title.clone(), // or root_layer_ogc.title or url suffix,
            description: "".to_string(),       // make configurable
            attribution: Attribution {
                title: None,
                online_resource: None,
            },
            item_abstract: "".to_string(), // caps.service.abstract_.clone(),
            keywords,
            online_resource: url.clone(),
            contact: Contact {
                person: "".to_string(), // "Service.ContactInformation.ContactPersonPrimary.ContactPerson"
                organization: "".to_string(), // "Service.ContactInformation.ContactPersonPrimary.ContactOrganization"
                position: "".to_string(),     // "Service.ContactInformation.ContactPosition"
                phone: "".to_string(),        // "Service.ContactInformation.ContactVoiceTelephone"
                email: "".to_string(), // "Service.ContactInformation.ContactElectronicMailAddress"
            },
            // format: configItem.format;
            available_formats: caps
                .capability
                .request
                .get_map
                .as_ref()
                .map(|r| r.format.clone())
                .unwrap_or(vec![]),
            // tiled: configItem.tiled;
            version: "1.3.0".to_string(), // configItem.version ? configItem.version : config.defaultWMSVersion
            info_formats: caps
                .capability
                .request
                .get_feature_info
                .as_ref()
                .map(|r| r.format.clone())
                .unwrap_or(vec![]),
            bbox: bbox.clone(),
            initial_bbox: bbox,
            // scales:  configItem.scales;
            // printScales:  configItem.printScales;
            print_resolutions: vec![300], // make configurable
            // printGrid:  configItem.printGrid;
            sublayers,
            expanded: true,
            external_layers: vec![], // make configurable
            background_layers: THEMES_JSON.themes.items[0].background_layers.clone(), // make configurable
            search_providers: vec!["coordinates".to_string()], // make configurable
            additional_mouse_crs: vec![],                      // make configurable
            map_crs: map_crs.unwrap_or("EPSG:3857".to_string()),
            drawing_order: vec![], // (capabilities.Capability.LayerDrawingOrder || "").split(",").map(title => title in titleNameMap ? titleNameMap[title] : title);
            legend_url: format!("{}?", url),
            feature_info_url: format!("{}?", url),
            print_url: format!("{}?", url),
            skip_empty_feature_attributes: true,
            edit_config: None,
            thumbnail: "img/mapthumbs/default.jpg".to_string(),
        };
        Some(theme)
    }
}

fn parse_layer_tree(ogc_layers: &Vec<ogc::Layer>) -> Vec<Layer> {
    let layers: Vec<Layer> = ogc_layers
        .iter()
        // skip layers without geometry
        //TODO: layer.$.geometryType == "WKBNoGeometry" || layer.$.geometryType == "NoGeometry") {
        .map(|l| {
            let sublayers = parse_layer_tree(&l.layers);
            let expanded = if sublayers.len() == 0 {
                None
            } else {
                l.expanded
            };
            let sublayers = if sublayers.len() == 0 {
                None
            } else {
                Some(sublayers)
            };

            let layer = Layer {
                name: l.name.as_ref().unwrap_or(&"".to_string()).clone(),
                title: l.title.as_ref().unwrap_or(&"".to_string()).clone(),
                visibility: l.visible.or(Some(true)),
                queryable: l.queryable,
                display_field: l.display_field.as_ref().map(|c| c.clone()),
                attribution: l.attribution.as_ref().map(|attr| Attribution {
                    title: Some(attr.title.clone()),
                    online_resource: Some(attr.online_resource.href.clone()),
                }),
                data_url: l.data_url.as_ref().map(|c| c.online_resource.href.clone()),
                metadata_url: l
                    .metadata_url
                    .as_ref()
                    .map(|c| c.online_resource.href.clone()),
                opacity: l.opacity.map(|v| (v * 255.0).round() as u8).or(Some(255)),
                keywords: Some("countries,political".to_string()), // TODO: Layer.KeywordList.Keyword
                bbox: l.ex_geographic_bounding_box.as_ref().map(|c| Bbox {
                    crs: "EPSG:4326".to_string(),
                    bounds: vec![
                        c.west_bound_longitude,
                        c.south_bound_latitude,
                        c.east_bound_longitude,
                        c.north_bound_latitude,
                    ],
                }),
                max_scale: l.min_scale_denominator.map(|v| v.round() as i64),
                min_scale: l.max_scale_denominator.map(|v| v.round() as i64),
                mutually_exclusive: l.mutually_exclusive,
                sublayers,
                expanded,
            };
            layer
        })
        .collect();
    layers
}

lazy_static! {
    static ref THEMES_JSON: ThemesJson = ThemesJson {
    themes: Themes {
        title: "root".to_string(),
        subdirs: vec![],
        items: vec![
            Theme {
                url: "http://127.0.0.1:8080/wms/qgs/ne".to_string(),
                id: "ne".to_string(),
                title: "Natural Earth".to_string(),
                description: "".to_string(),
                attribution: Attribution {
                    title: Some(
                        "Natural Earth".to_string(),
                    ),
                    online_resource: Some(
                        "".to_string(),
                    ),
                },
                item_abstract: "".to_string(),
                keywords: "".to_string(),
                online_resource: "http://127.0.0.1:8080/wms/qgs/ne".to_string(),
                contact: Contact {
                    person: "".to_string(),
                    organization: "".to_string(),
                    position: "".to_string(),
                    phone: "".to_string(),
                    email: "".to_string(),
                },
                available_formats: vec![
                    "image/jpeg".to_string(),
                    "image/png".to_string(),
                    "image/png; mode=16bit".to_string(),
                    "image/png; mode=8bit".to_string(),
                    "image/png; mode=1bit".to_string(),
                    "application/dxf".to_string(),
                ],
                version: "1.3.0".to_string(),
                info_formats: vec![
                    "text/plain".to_string(),
                    "text/html".to_string(),
                    "text/xml".to_string(),
                    "application/vnd.ogc.gml".to_string(),
                    "application/vnd.ogc.gml/3.1.1".to_string(),
                    "application/json".to_string(),
                    "application/geo+json".to_string(),
                ],
                bbox: Bbox {
                    crs: "EPSG:4326".to_string(),
                    bounds: vec![
                        -179.999926,
                        -89.999996,
                        179.999927,
                        89.999996,
                    ],
                },
                initial_bbox: Bbox {
                    crs: "EPSG:3857".to_string(),
                    bounds: vec![
                        -1000000.0,
                        4000000.0,
                        3000000.0,
                        8000000.0,
                    ],
                },
                print_resolutions: vec![
                    300,
                ],
                sublayers: vec![
                    Layer {
                        name: "ne".to_string(),
                        title: "Natural Earth".to_string(),
                        visibility: Some(
                            true,
                        ),
                        queryable: Some(
                            true,
                        ),
                        display_field: Some(
                            "z_name".to_string(),
                        ),
                        attribution: Some(
                            Attribution {
                                title: Some(
                                    "Natural Earth".to_string(),
                                ),
                                online_resource: Some(
                                    "https://www.naturalearthdata.com/".to_string(),
                                ),
                            },
                        ),
                        data_url: Some(
                            "https://www.naturalearthdata.com/".to_string(),
                        ),
                        metadata_url: Some(
                            "https://www.naturalearthdata.com/about/".to_string(),
                        ),
                        opacity: Some(
                            255,
                        ),
                        keywords: Some(
                            "countries,political".to_string(),
                        ),
                        bbox: Some(
                            Bbox {
                                crs: "EPSG:4326".to_string(),
                                bounds: vec![
                                    -177.228623,
                                    -80.516517,
                                    178.519502,
                                    73.348998,
                                ],
                            },
                        ),
                        max_scale: None,
                        min_scale: None,
                        mutually_exclusive: None,
                        sublayers: None,
                        expanded: None,
                    },
                    Layer {
                        name: "state".to_string(),
                        title: "state".to_string(),
                        visibility: Some(
                            true,
                        ),
                        queryable: Some(
                            true,
                        ),
                        display_field: Some(
                            "name".to_string(),
                        ),
                        attribution: None,
                        data_url: None,
                        metadata_url: None,
                        opacity: Some(
                            255,
                        ),
                        keywords: None,
                        bbox: Some(
                            Bbox {
                                crs: "EPSG:4326".to_string(),
                                bounds: vec![
                                    -139.060207,
                                    -39.201702,
                                    153.506568,
                                    78.686917,
                                ],
                            },
                        ),
                        max_scale: None,
                        min_scale: None,
                        mutually_exclusive: None,
                        sublayers: None,
                        expanded: None,
                    },
                    Layer {
                        name: "country".to_string(),
                        title: "country".to_string(),
                        visibility: Some(
                            true,
                        ),
                        queryable: Some(
                            true,
                        ),
                        display_field: Some(
                            "name".to_string(),
                        ),
                        attribution: None,
                        data_url: None,
                        metadata_url: None,
                        opacity: Some(
                            255,
                        ),
                        keywords: None,
                        bbox: Some(
                            Bbox {
                                crs: "EPSG:4326".to_string(),
                                bounds: vec![
                                    -179.999926,
                                    -89.501388,
                                    179.999926,
                                    83.634081,
                                ],
                            },
                        ),
                        max_scale: None,
                        min_scale: None,
                        mutually_exclusive: None,
                        sublayers: None,
                        expanded: None,
                    },
                    Layer {
                        name: "geo-lines".to_string(),
                        title: "geo-lines".to_string(),
                        visibility: None,
                        queryable: None,
                        display_field: None,
                        attribution: None,
                        data_url: None,
                        metadata_url: None,
                        opacity: None,
                        keywords: None,
                        bbox: None,
                        max_scale: None,
                        min_scale: None,
                        mutually_exclusive: Some(
                            false,
                        ),
                        sublayers: Some(
                            vec![
                                Layer {
                                    name: "ne_10m_geographic_lines".to_string(),
                                    title: "ne_10m_geographic_lines".to_string(),
                                    attribution: None,
                                    data_url: None,
                                    metadata_url: None,
                                    keywords: None,
                                    visibility: Some(false),
                                    queryable: Some(true),
                                    display_field: Some("name".to_string()),
                                    opacity: Some(255),
                                    min_scale: Some(0),
                                    max_scale: Some(5000000),
                                    bbox: Some(Bbox {
                                        crs: "EPSG:4326".to_string(),
                                        bounds: vec![
                                            -179.999926,
                                            -89.999996,
                                            179.999926,
                                            89.999996,
                                        ],
                                    }),
                                    mutually_exclusive: None,
                                    sublayers: None,
                                    expanded: None,
                                },
                                Layer {
                                    name: "ne_50m_geographic_lines".to_string(),
                                    title: "ne_50m_geographic_lines".to_string(),
                                    attribution: None,
                                    data_url: None,
                                    metadata_url: None,
                                    keywords: None,
                                    visibility: Some(false),
                                    queryable: Some(true),
                                    display_field: Some("name".to_string()),
                                    opacity: Some(255),
                                    min_scale: Some(5000000),
                                    max_scale: Some(100000000),
                                    bbox: Some(Bbox {
                                        crs: "EPSG:4326".to_string(),
                                        bounds: vec![
                                            -179.999926,
                                            -89.999996,
                                            179.999926,
                                            89.999996,
                                        ],
                                    }),
                                    mutually_exclusive: None,
                                    sublayers: None,
                                    expanded: None,
                                },
                            ],
                        ),
                        expanded: Some(
                            true,
                        ),
                    },
                ],
                expanded: true,
                external_layers: vec![],
                background_layers: vec![
                    ThemeBackgroundLayer {
                        name: "bluemarble".to_string(),
                        print_layer: "bluemarble_bg".to_string(),
                        visibility: false,
                    },
                    ThemeBackgroundLayer {
                        name: "mapnik".to_string(),
                        print_layer: "osm_bg".to_string(),
                        visibility: false,
                    },
                ],
                search_providers: vec![
                    "coordinates".to_string(),
                ],
                additional_mouse_crs: vec![],
                map_crs: "EPSG:3857".to_string(),
                drawing_order: vec![
                    "ne_50m_geographic_lines".to_string(),
                    "ne_10m_geographic_lines".to_string(),
                    "country".to_string(),
                    "state".to_string(),
                    "ne".to_string(),
                ],
                legend_url: "http://127.0.0.1:8080/wms/qgs/ne?".to_string(),
                feature_info_url: "http://127.0.0.1:8080/wms/qgs/ne?".to_string(),
                print_url: "http://127.0.0.1:8080/wms/qgs/ne?".to_string(),
                skip_empty_feature_attributes: true,
                edit_config: None,
                thumbnail: "img/mapthumbs/default.jpg".to_string(),
            },
        ],
        default_theme: "ne".to_string(),
        default_scales: vec![
            100000000,
            50000000,
            25000000,
            10000000,
            4000000,
            2000000,
            1000000,
            400000,
            200000,
            80000,
            40000,
            20000,
            10000,
            8000,
            6000,
            4000,
            2000,
            1000,
            500,
            250,
            100,
        ],
        default_print_grid: vec![
            GridInterval {
                s: 10000000,
                x: 1000000,
                y: 1000000,
            },
            GridInterval {
                s: 1000000,
                x: 100000,
                y: 100000,
            },
            GridInterval {
                s: 100000,
                x: 10000,
                y: 10000,
            },
            GridInterval {
                s: 10000,
                x: 1000,
                y: 1000,
            },
            GridInterval {
                s: 1000,
                x: 100,
                y: 100,
            },
            GridInterval {
                s: 100,
                x: 10,
                y: 10,
            },
        ],
        background_layers: vec![
            BackgroundLayer {
                name: "mapnik".to_string(),
                title: "Open Street Map".to_string(),
                background_layer_type: "osm".to_string(),
                source: Some(
                    "osm".to_string(),
                ),
                thumbnail: "img/mapthumbs/mapnik.jpg".to_string(),
                attribution: Attribution {
                    title: Some(
                        "OpenStreetMap contributors".to_string(),
                    ),
                    online_resource: Some(
                        "https://www.openstreetmap.org/copyright".to_string(),
                    ),
                },
                url: None,
                tile_matrix_prefix: None,
                tile_matrix_set: None,
                origin_x: None,
                origin_y: None,
                projection: None,
                resolutions: None,
                tile_size: None,
            },
            BackgroundLayer {
                name: "bluemarble".to_string(),
                title: "Blue Marble".to_string(),
                background_layer_type: "wmts".to_string(),
                source: None,
                thumbnail: "img/mapthumbs/default.jpg".to_string(),
                attribution: Attribution {
                    title: None,
                    online_resource: None,
                },
                url: Some(
                    "http://gibs.earthdata.nasa.gov/wmts/epsg3857/best/BlueMarble_ShadedRelief/default/{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}.jpeg".to_string(),
                ),
                tile_matrix_prefix: Some(
                    "".to_string(),
                ),
                tile_matrix_set: Some(
                    "GoogleMapsCompatible_Level8".to_string(),
                ),
                origin_x: Some(
                    -20037508.34278925,
                ),
                origin_y: Some(
                    20037508.34278925,
                ),
                projection: Some(
                    "EPSG:3857".to_string(),
                ),
                resolutions: Some(
                    vec![
                        156543.03390625,
                        78271.516953125,
                        39135.7584765625,
                        19567.87923828125,
                        9783.939619140623,
                        4891.9698095703125,
                        2445.984904785156,
                        1222.992452392578,
                    ],
                ),
                tile_size: Some(
                    [
                        256,
                        256,
                    ],
                ),
            },
        ],
        default_wms_version: "1.3.0".to_string(),
    },
};
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize() {
        let themes_json_str = r##"{
  "themes": {
    "title": "root",
    "subdirs": [],
    "items": [
      {
        "url": "http://127.0.0.1:8080/wms/qgs/ne",
        "id": "ne",
        "title": "Natural Earth",
        "description": "",
        "attribution": {
          "Title": "Natural Earth",
          "OnlineResource": ""
        },
        "abstract": "",
        "keywords": "",
        "onlineResource": "http://127.0.0.1:8080/wms/qgs/ne",
        "contact": {
          "person": "",
          "organization": "",
          "position": "",
          "phone": "",
          "email": ""
        },
        "availableFormats": [
          "image/jpeg",
          "image/png",
          "image/png; mode=16bit",
          "image/png; mode=8bit",
          "image/png; mode=1bit",
          "application/dxf"
        ],
        "version": "1.3.0",
        "infoFormats": [
          "text/plain",
          "text/html",
          "text/xml",
          "application/vnd.ogc.gml",
          "application/vnd.ogc.gml/3.1.1",
          "application/json",
          "application/geo+json"
        ],
        "bbox": {
          "crs": "EPSG:4326",
          "bounds": [
            -179.999926,
            -89.999996,
            179.999927,
            89.999996
          ]
        },
        "initialBbox": {
          "crs": "EPSG:3857",
          "bounds": [
            -1000000.0,
            4000000.0,
            3000000.0,
            8000000.0
          ]
        },
        "printResolutions": [
          300
        ],
        "sublayers": [
          {
            "name": "ne",
            "title": "Natural Earth",
            "visibility": true,
            "queryable": true,
            "displayField": "z_name",
            "attribution": {
              "Title": "Natural Earth",
              "OnlineResource": "https://www.naturalearthdata.com/"
            },
            "dataUrl": "https://www.naturalearthdata.com/",
            "metadataUrl": "https://www.naturalearthdata.com/about/",
            "opacity": 255,
            "keywords": "countries,political",
            "bbox": {
              "crs": "EPSG:4326",
              "bounds": [
                -177.228623,
                -80.516517,
                178.519502,
                73.348998
              ]
            }
          },
          {
            "name": "state",
            "title": "state",
            "visibility": true,
            "queryable": true,
            "displayField": "name",
            "opacity": 255,
            "bbox": {
              "crs": "EPSG:4326",
              "bounds": [
                -139.060207,
                -39.201702,
                153.506568,
                78.686917
              ]
            }
          },
          {
            "name": "country",
            "title": "country",
            "visibility": true,
            "queryable": true,
            "displayField": "name",
            "opacity": 255,
            "bbox": {
              "crs": "EPSG:4326",
              "bounds": [
                -179.999926,
                -89.501388,
                179.999926,
                83.634081
              ]
            }
          },
          {
            "name": "geo-lines",
            "title": "geo-lines",
            "visibility": null,
            "queryable": null,
            "displayField": null,
            "opacity": null,
            "mutuallyExclusive": false,
            "sublayers": [
              {
                "name": "ne_10m_geographic_lines",
                "title": "ne_10m_geographic_lines",
                "visibility": false,
                "queryable": true,
                "displayField": "name",
                "opacity": 255,
                "minScale": 0,
                "maxScale": 5000000,
                "bbox": {
                  "crs": "EPSG:4326",
                  "bounds": [
                    -179.999926,
                    -89.999996,
                    179.999926,
                    89.999996
                  ]
                }
              },
              {
                "name": "ne_50m_geographic_lines",
                "title": "ne_50m_geographic_lines",
                "visibility": false,
                "queryable": true,
                "displayField": "name",
                "opacity": 255,
                "minScale": 5000000,
                "maxScale": 100000000,
                "bbox": {
                  "crs": "EPSG:4326",
                  "bounds": [
                    -179.999926,
                    -89.999996,
                    179.999926,
                    89.999996
                  ]
                }
              }
            ],
            "expanded": true
          }
        ],
        "expanded": true,
        "externalLayers": [],
        "backgroundLayers": [
          {
            "name": "bluemarble",
            "printLayer": "bluemarble_bg",
            "visibility": false
          },
          {
            "name": "mapnik",
            "printLayer": "osm_bg",
            "visibility": false
          }
        ],
        "searchProviders": [
          "coordinates"
        ],
        "additionalMouseCrs": [],
        "mapCrs": "EPSG:3857",
        "drawingOrder": [
          "ne_50m_geographic_lines",
          "ne_10m_geographic_lines",
          "country",
          "state",
          "ne"
        ],
        "legendUrl": "http://127.0.0.1:8080/wms/qgs/ne?",
        "featureInfoUrl": "http://127.0.0.1:8080/wms/qgs/ne?",
        "printUrl": "http://127.0.0.1:8080/wms/qgs/ne?",
        "skipEmptyFeatureAttributes": true,
        "editConfig": null,
        "thumbnail": "img/mapthumbs/default.jpg"
      }
    ],
    "defaultTheme": "ne",
    "defaultScales": [
      100000000,
      50000000,
      25000000,
      10000000,
      4000000,
      2000000,
      1000000,
      400000,
      200000,
      80000,
      40000,
      20000,
      10000,
      8000,
      6000,
      4000,
      2000,
      1000,
      500,
      250,
      100
    ],
    "defaultPrintGrid": [
      {
        "s": 10000000,
        "x": 1000000,
        "y": 1000000
      },
      {
        "s": 1000000,
        "x": 100000,
        "y": 100000
      },
      {
        "s": 100000,
        "x": 10000,
        "y": 10000
      },
      {
        "s": 10000,
        "x": 1000,
        "y": 1000
      },
      {
        "s": 1000,
        "x": 100,
        "y": 100
      },
      {
        "s": 100,
        "x": 10,
        "y": 10
      }
    ],
    "backgroundLayers": [
      {
        "name": "mapnik",
        "title": "Open Street Map",
        "type": "osm",
        "source": "osm",
        "thumbnail": "img/mapthumbs/mapnik.jpg",
        "attribution": {
          "Title": "OpenStreetMap contributors",
          "OnlineResource": "https://www.openstreetmap.org/copyright"
        },
        "url": null,
        "tileMatrixPrefix": null,
        "tileMatrixSet": null,
        "originX": null,
        "originY": null,
        "projection": null,
        "resolutions": null,
        "tileSize": null
      },
      {
        "name": "bluemarble",
        "title": "Blue Marble",
        "type": "wmts",
        "source": null,
        "thumbnail": "img/mapthumbs/default.jpg",
        "attribution": {
          "Title": null,
          "OnlineResource": null
        },
        "url": "http://gibs.earthdata.nasa.gov/wmts/epsg3857/best/BlueMarble_ShadedRelief/default/{TileMatrixSet}/{TileMatrix}/{TileRow}/{TileCol}.jpeg",
        "tileMatrixPrefix": "",
        "tileMatrixSet": "GoogleMapsCompatible_Level8",
        "originX": -20037508.34278925,
        "originY": 20037508.34278925,
        "projection": "EPSG:3857",
        "resolutions": [
          156543.03390625,
          78271.516953125,
          39135.7584765625,
          19567.87923828125,
          9783.939619140623,
          4891.9698095703125,
          2445.984904785156,
          1222.992452392578
        ],
        "tileSize": [
          256,
          256
        ]
      }
    ],
    "defaultWMSVersion": "1.3.0"
  }
}"##;

        let jsonstr = serde_json::to_string_pretty(&THEMES_JSON as &ThemesJson).unwrap();
        println!("{}", jsonstr);
        assert_eq!(jsonstr, themes_json_str);

        // Deserialize:
        // let themes_json: ThemesJson = serde_json::from_str(&themes_json_str).unwrap();
        // println!("{:#?}", themes_json);
        // assert_eq!(themes_json.themes.title, THEMES_JSON.themes.title);
    }
}

use crate::qgis_plugins::*;
use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest, Result};
use bbox_common::app_dir;
use bbox_common::config::config_error_exit;
use log::{info, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::tempfile;

#[derive(Deserialize, Default, Debug)]
pub struct FileserverCfg {
    #[serde(rename = "static", default)]
    pub static_: Vec<StaticDirCfg>,
    #[serde(default)]
    pub repo: Vec<QgisPluginRepoCfg>,
}

#[derive(Deserialize, Debug)]
pub struct StaticDirCfg {
    pub path: String,
    pub dir: String,
}

impl FileserverCfg {
    pub fn from_config() -> Self {
        let config = bbox_common::config::app_config();
        if config.find_value("fileserver").is_ok() {
            config
                .extract_inner("fileserver")
                .map_err(|err| config_error_exit(err))
                .unwrap()
        } else {
            Default::default()
        }
    }
}

type PluginIndex = HashMap<String, Vec<PathBuf>>;

fn req_baseurl(req: &HttpRequest) -> String {
    let conninfo = req.connection_info();
    format!("{}://{}", conninfo.scheme(), conninfo.host())
}

async fn plugin_xml(plugins_index: web::Data<PluginIndex>, req: HttpRequest) -> Result<NamedFile> {
    // http://localhost:8080/qgis/plugins.xml -> http://localhost:8080/plugins/qgis/
    let url = format!(
        "{}/plugins{}",
        req_baseurl(&req),
        Path::new(req.path())
            .parent()
            .expect("invalid req.path")
            .to_str()
            .expect("invalid req.path")
    );
    let zips = plugins_index
        .get(req.path())
        .expect("zip file list missing");
    let plugins = plugin_metadata(zips);
    let xml = render_plugin_xml(&plugins, &url);
    let mut file = tempfile()?;
    file.write_all(xml.as_bytes())?;
    Ok(NamedFile::from_file(file, "plugin.xml")?)
}

pub fn init_service() -> PluginIndex {
    let static_cfg = FileserverCfg::from_config();

    let mut plugins_index = PluginIndex::new();
    for repo in &static_cfg.repo {
        let dir = app_dir(&repo.dir);
        if Path::new(&dir).is_dir() {
            info!("Serving QGIS plugin repository from directory '{}'", &dir);
            let plugins = plugin_files(&dir);
            plugins_index.insert(format!("/{}/plugins.xml", repo.path), plugins);
        } else {
            warn!("QGIS plugin repository file directory '{}' not found", &dir);
        }
    }
    plugins_index
}

pub fn register(cfg: &mut web::ServiceConfig, plugins_index: &PluginIndex) {
    let static_cfg = FileserverCfg::from_config();

    for static_dir in &static_cfg.static_ {
        let dir = app_dir(&static_dir.dir);
        if Path::new(&dir).is_dir() {
            info!("Serving static files from directory '{}'", &dir);
            cfg.service(Files::new(&static_dir.path, &dir));
        } else {
            warn!("Static file directory '{}' not found", &dir);
        }
    }

    cfg.data(plugins_index.clone());

    for repo in &static_cfg.repo {
        let dir = app_dir(&repo.dir);
        if Path::new(&dir).is_dir() {
            // info!("Serving QGIS plugin repository from directory '{}'", &dir);
            cfg.service(Files::new(
                "/qgis/static",
                app_dir("bbox-file-server/src/static"), // TODO: RustEmbed !
            ))
            .route(
                &format!("/{}/plugins.xml", repo.path),
                web::get().to(plugin_xml),
            )
            // TODO: same prefix not possible?
            .service(Files::new(&format!("/plugins/{}", repo.path), &dir));
        } else {
            // warn!("QGIS plugin repository file directory '{}' not found", &dir);
        }
    }
}

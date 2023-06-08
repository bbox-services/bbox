use crate::config::FileserverCfg;
use crate::qgis_plugins::*;
use crate::service::{FileService, PluginIndex};
use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest, Result};
use bbox_common::app_dir;
use bbox_common::endpoints::{abs_req_baseurl, req_parent_path};
use bbox_common::service::CoreService;
use log::{info, warn};
use std::io::Write;
use std::path::Path;
use tempfile::tempfile;

async fn plugin_xml(plugins_index: web::Data<PluginIndex>, req: HttpRequest) -> Result<NamedFile> {
    // http://localhost:8080/qgis/plugins.xml -> http://localhost:8080/plugins/qgis/
    let url = format!("{}/plugins{}", abs_req_baseurl(&req), req_parent_path(&req));
    let zips = plugins_index
        .get(req.path())
        .expect("zip file list missing");
    let plugins = plugin_metadata(zips);
    let xml = render_plugin_xml(&plugins, &url);
    let mut file = tempfile()?;
    file.write_all(xml.as_bytes())?;
    Ok(NamedFile::from_file(file, "plugin.xml")?)
}

impl FileService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        let static_cfg = FileserverCfg::from_config();

        for static_dir in &static_cfg.static_ {
            let dir = app_dir(&static_dir.dir);
            if Path::new(&dir).is_dir() {
                info!("Serving static files from directory '{dir}'");
                cfg.service(Files::new(&static_dir.path, &dir));
            } else {
                warn!("Static file directory '{dir}' not found");
            }
        }

        cfg.app_data(web::Data::new(self.plugins_index.clone()));

        for repo in &static_cfg.repo {
            let dir = app_dir(&repo.dir);
            if Path::new(&dir).is_dir() {
                info!("Serving QGIS plugin repository from directory '{dir}'");
                cfg.service(Files::new(
                    &format!("/{}/static", repo.path),
                    app_dir("bbox-file-server/src/static"), // TODO: RustEmbed !
                ))
                .route(
                    &format!("/{}/plugins.xml", repo.path),
                    web::get().to(plugin_xml),
                )
                // TODO: same prefix not possible?
                .service(Files::new(&format!("/plugins/{}", repo.path), &dir));
            } else {
                // warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }
    }
}

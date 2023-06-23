use crate::config::AssetserverCfg;
use crate::qgis_plugins::*;
use crate::runtime_templates::RuntimeTemplates;
use crate::service::{AssetService, PluginIndex};
use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use bbox_core::app_dir;
use bbox_core::endpoints::{abs_req_baseurl, req_parent_path};
use bbox_core::service::CoreService;
use log::{info, warn};
use minijinja::context;
use std::io::Write;
use std::path::Path;
use tempfile::tempfile;

async fn templates(
    envs: web::Data<RuntimeTemplates>,
    template: web::Path<(String, String)>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let path = Path::new(req.path())
        .parent()
        .expect("invalid req.path")
        .parent()
        .expect("invalid req.path")
        .to_str()
        .expect("invalid req.path")
        .to_string();
    let (stem, param) = template.into_inner();
    let name = format!("{stem}.html");
    let env = envs.get(&path).unwrap();
    let tmpl = env.get_template(&name).unwrap();
    let out = tmpl
        .render(context!(param => param))
        .expect("Template render failed");
    Ok(HttpResponse::Ok().content_type("text/html").body(out))
}

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

impl AssetService {
    pub(crate) fn register(&self, cfg: &mut web::ServiceConfig, _core: &CoreService) {
        let service_cfg = AssetserverCfg::from_config();

        for static_dir in &service_cfg.static_ {
            let dir = app_dir(&static_dir.dir);
            if Path::new(&dir).is_dir() {
                info!(
                    "Serving static files from directory '{dir}' on '{}'",
                    &static_dir.path
                );
                cfg.service(Files::new(&static_dir.path, &dir));
            } else {
                warn!("Static file directory '{dir}' not found");
            }
        }

        let mut template_envs = RuntimeTemplates::new();
        for template_dir in &service_cfg.template {
            let dir = app_dir(&template_dir.dir);
            if Path::new(&dir).is_dir() {
                let dest = &template_dir.path;
                info!("Serving template files from directory '{dir}' on '{dest}'");
                template_envs.add(&dir, dest);
                cfg.route(
                    &format!("{dest}/{{name}}/{{param}}"),
                    web::get().to(templates),
                );
            } else {
                warn!("Template file directory '{dir}' not found");
            }
        }
        cfg.app_data(web::Data::new(template_envs));

        cfg.app_data(web::Data::new(self.plugins_index.clone()));

        for repo in &service_cfg.repo {
            let dir = app_dir(&repo.dir);
            if Path::new(&dir).is_dir() {
                let xmldir = format!("{}/plugins.xml", repo.path);
                info!("Serving QGIS plugin repository from directory '{dir}' on '{xmldir}'");
                cfg.service(Files::new(
                    &format!("{}/static", repo.path),
                    app_dir("bbox-asset-server/src/static"), // TODO: RustEmbed !
                ))
                .route(&xmldir, web::get().to(plugin_xml))
                // TODO: same prefix not possible?
                .service(Files::new(&format!("/plugins{}", repo.path), &dir));
            } else {
                // warn!("QGIS plugin repository file directory '{dir}' not found");
            }
        }
    }
}

use bbox_core::{file_search, templates::create_env_embedded};
use configparser::ini::Ini;
use minijinja::{context, Environment};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::result::ZipResult;

#[derive(Deserialize, Debug)]
pub struct QgisPluginRepoCfg {
    /// endpoint path for publishing
    pub path: String,
    /// template file directory
    pub dir: String,
}

#[derive(Serialize)]
pub struct Plugins {
    #[serde(default)]
    plugins: Vec<Plugin>,
}

#[derive(Serialize, Debug)]
pub struct Plugin {
    file_name: String,
    name: String,
    qgis_minimum_version: String,
    qgis_maximum_version: String,
    description: String,
    about: Option<String>,
    version: String,
    author: String,
    email: String,
    changelog: Option<String>,
    tags: Option<String>,
    homepage: Option<String>,
    tracker: Option<String>,
    repository: Option<String>,
    icon: Option<String>,
    experimental: Option<String>,
    deprecated: Option<String>,
}

#[derive(RustEmbed)]
#[folder = "src/templates/"]
struct Templates;

static TEMPLATE_ENV: Lazy<Environment<'static>> = Lazy::new(|| create_env_embedded(&Templates));

pub fn render_plugin_xml(plugins: &Plugins, url: &str) -> String {
    let template = TEMPLATE_ENV
        .get_template("plugins.xml")
        .expect("couln't load template `plugins.xml`");
    template
        .render(context!(plugins => plugins.plugins, url => url))
        .expect("Plugin render failed")
}

fn read_metadata(fname: &PathBuf) -> ZipResult<Plugin> {
    fn get_entry(ini: &Ini, key: &str) -> String {
        ini.get("general", key)
            .unwrap_or(format!("{} missing", key))
    }
    let zipfile = std::fs::File::open(fname)?;
    let mut archive = zip::ZipArchive::new(zipfile)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        if Path::new(&outpath).file_name().and_then(OsStr::to_str) == Some("metadata.txt") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            // This Ini parser returns first line of multi-line strings (others return error)
            let mut ini = Ini::new();
            ini.read(contents.clone()).ok();
            let meta = Plugin {
                file_name: fname
                    .file_name()
                    .and_then(OsStr::to_str)
                    .expect("invalid file_name")
                    .to_string(),
                name: get_entry(&ini, "name"),
                qgis_minimum_version: get_entry(&ini, "qgisMinimumVersion"),
                qgis_maximum_version: get_entry(&ini, "qgisMaximumVersion"),
                description: get_entry(&ini, "description"),
                about: ini.get("general", "about"),
                version: get_entry(&ini, "version"),
                author: get_entry(&ini, "author"),
                email: get_entry(&ini, "email"),
                changelog: None,
                tags: ini.get("general", "tags"),
                homepage: ini.get("general", "homepage"),
                tracker: ini.get("general", "tracker"),
                repository: ini.get("general", "repository"),
                icon: ini.get("general", "icon"),
                experimental: ini.get("general", "experimental"),
                deprecated: ini.get("general", "deprecated"),
            };
            return Ok(meta);
        }
    }
    Err(zip::result::ZipError::FileNotFound)
}

pub fn plugin_metadata(plugin_fnames: &[PathBuf]) -> Plugins {
    let plugins: Vec<Plugin> = plugin_fnames
        .iter()
        .filter_map(|fname| read_metadata(fname).ok())
        .collect();
    Plugins { plugins }
}

pub fn plugin_files(repo_path: &str) -> Vec<PathBuf> {
    file_search::search(repo_path, "*.zip")
}

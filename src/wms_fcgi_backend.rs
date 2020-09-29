use crate::fcgi_process::{FcgiClientHandler, FcgiProcess};
use crate::file_search;
use log::info;
use std::env;
use std::path::Path;

pub struct FcgiBackend {
    exe_path: String,
    base_dir: String,
}

impl FcgiBackend {
    fn detect_fcgi(locations: Vec<&str>) -> Option<FcgiBackend> {
        if let Some(exe_path) = find_exe(locations) {
            Some(FcgiBackend {
                exe_path,
                base_dir: ".".to_string(),
            })
        } else {
            None
        }
    }

    async fn spawn_process(&self) -> std::io::Result<FcgiProcess> {
        let process = FcgiProcess::spawn(&self.exe_path).await;
        process
    }

    pub async fn spawn_backend(backend: &dyn FcgiBackendType) -> Option<FcgiProcess> {
        if let Some(backend) = FcgiBackend::detect_fcgi(backend.exe_locations()) {
            if let Ok(process) = backend.spawn_process().await {
                return Some(process);
            }
        }
        None
    }
}

fn find_exe(locations: Vec<&str>) -> Option<String> {
    locations
        .iter()
        .find(|&&c| Path::new(c).is_file())
        .map(|&c| c.to_string())
}

pub trait FcgiBackendType {
    fn name(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
}

pub struct QgisFcgiBackend;

impl FcgiBackendType for QgisFcgiBackend {
    fn name(&self) -> &'static str {
        "QGIS Server"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/qgis_mapserv.fcgi"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["qgs", "qgz"]
    }
}

pub struct UmnFcgiBackend;

impl FcgiBackendType for UmnFcgiBackend {
    fn name(&self) -> &'static str {
        "UMN Mapserver"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/mapserv"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["map"]
    }
}

pub async fn init_backends(
) -> std::io::Result<(Vec<FcgiProcess>, Vec<(FcgiClientHandler, String, String)>)> {
    let mut processes = Vec::new();
    let mut handlers = Vec::new();
    let curdir = env::current_dir()
        .expect("current_dir unkown")
        .canonicalize()?;
    let backends: Vec<&dyn FcgiBackendType> = vec![&QgisFcgiBackend {}, &UmnFcgiBackend {}];
    for backend in backends {
        if let Some(process) = FcgiBackend::spawn_backend(backend).await {
            info!("{} FCGI process started", backend.name());
            for ending in backend.project_files() {
                info!(
                    "Searching project files with project_scan_basedir: {}",
                    curdir.to_str().expect("Invalid UTF-8 path name")
                );
                let files = file_search::search(&curdir, &format!("*.{}", ending));
                info!("Found {} file(s) matching *.{}", files.len(), ending);
                let path = format!("/wms/{}", ending);
                info!("Registering WMS endpoint {}", &path);
                handlers.push((process.handler(), path, ending.to_string()));
            }
            processes.push(process);
        }
    }
    Ok((processes, handlers))
}

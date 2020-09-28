use crate::fcgi_process::FcgiProcess;
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
    fn default_url_prefix(&self) -> &'static str;
    fn exe_locations(&self) -> Vec<&'static str>;
    fn project_files(&self) -> Vec<&'static str>;
}

pub struct QgisFcgiBackend;

impl FcgiBackendType for QgisFcgiBackend {
    fn name(&self) -> &'static str {
        "QGIS Server"
    }
    fn default_url_prefix(&self) -> &'static str {
        "qgis"
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
    fn default_url_prefix(&self) -> &'static str {
        "umn"
    }
    fn exe_locations(&self) -> Vec<&'static str> {
        vec!["/usr/lib/cgi-bin/mapserv"]
    }
    fn project_files(&self) -> Vec<&'static str> {
        vec!["map"]
    }
}

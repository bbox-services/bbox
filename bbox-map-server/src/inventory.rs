use crate::wms_capabilities::*;
use serde::Serialize;
use serde_xml_rs::from_reader;

#[derive(Serialize, Clone, Default, Debug)]
pub struct Inventory {
    pub wms_services: Vec<WmsService>,
    pub public_server_url: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct WmsService {
    pub id: String,
    /// WMS base path like `/qgis/ne`
    pub wms_path: String,
    pub cap_type: CapType,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
pub enum CapType {
    Ogc,
    Qgis,
}

impl Inventory {
    pub fn base_url(&self) -> &str {
        self.public_server_url.as_deref().unwrap_or("/")
    }
}

impl WmsService {
    fn _project(&self) -> &str {
        self.wms_path.split('/').last().expect("invalid wms_path")
    }
    #[allow(dead_code)]
    fn cap_request(&self) -> &str {
        match self.cap_type {
            CapType::Ogc => "GetCapabilities",
            CapType::Qgis => "GetProjectSettings",
        }
    }
    #[allow(dead_code)]
    pub fn url(&self, base_url: &str) -> String {
        format!("{}{}", base_url, self.wms_path)
    }
    #[allow(dead_code)]
    pub async fn capabilities(&self, base_url: &str) -> WmsCapabilities {
        let client = awc::Client::default();
        let mut response = client
            .get(format!(
                "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST={}",
                &self.url(base_url),
                self.cap_request()
            ))
            .send()
            .await
            .expect("GetCapabilities");

        let body = response.body().await.unwrap();
        let cap: WmsCapabilities = from_reader(body.as_ref()).unwrap();
        cap
    }
}

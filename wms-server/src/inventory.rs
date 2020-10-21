use crate::wms_capabilities::*;
use serde_xml_rs::from_reader;

#[derive(Clone, Debug)]
pub struct Inventory {
    pub wms_services: Vec<WmsService>,
}

#[derive(Clone, Debug)]
pub struct WmsService {
    /// WMS base path like `/wms/qgs/ne`
    pub wms_path: String,
    pub cap_type: CapType,
}

#[derive(Clone, Debug)]
pub enum CapType {
    Ogc,
    Qgis,
}

impl WmsService {
    fn _project(&self) -> &str {
        self.wms_path.split('/').last().expect("invalid wms_path")
    }

    fn cap_request(&self) -> &str {
        match self.cap_type {
            CapType::Ogc => "GetCapabilities",
            CapType::Qgis => "GetProjectSettings",
        }
    }

    pub async fn capabilities(&self, base_url: &str) -> WmsCapabilities {
        let client = awc::Client::default();
        let mut response = client
            .get(format!(
                "{}{}?SERVICE=WMS&VERSION=1.3.0&REQUEST={}",
                base_url,
                self.wms_path,
                self.cap_request()
            ))
            .send()
            .await
            .unwrap();

        let body = response.body().await.unwrap();
        let cap: WmsCapabilities = from_reader(body.as_ref()).unwrap();
        cap
    }
}

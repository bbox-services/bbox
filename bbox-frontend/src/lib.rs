pub mod endpoints;
#[cfg(feature = "qwc2")]
mod qwc2_config;

#[cfg(feature = "qwc2")]
pub use crate::qwc2_config::themes_json;

#[cfg(feature = "qwc2")]
pub use bbox_map_server::inventory::{Inventory as MapInventory, WmsService};

#[cfg(not(feature = "qwc2"))]
mod qwc2 {
    #[derive(serde::Serialize)]
    pub struct WmsService;

    pub struct MapInventory {
        pub wms_services: Vec<WmsService>,
    }

    pub async fn themes_json(_: &Vec<WmsService>, _: String, _: Option<&str>) -> String {
        unimplemented!()
    }
}

#[cfg(not(feature = "qwc2"))]
pub use qwc2::*;

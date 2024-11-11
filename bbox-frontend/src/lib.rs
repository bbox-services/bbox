pub mod endpoints;
#[cfg(feature = "map-server")]
mod qwc2_config;

#[cfg(feature = "map-server")]
pub use crate::qwc2_config::themes_json;

#[cfg(feature = "map-server")]
pub use bbox_map_server::inventory::{Inventory as MapInventory, WmsService};

#[cfg(not(feature = "map-server"))]
mod dummy_inventory {
    #[derive(serde::Serialize)]
    pub struct WmsService;

    #[derive(Default)]
    pub struct MapInventory {
        pub wms_services: Vec<WmsService>,
    }

    impl MapInventory {
        pub fn base_url(&self) -> &str {
            "/"
        }
    }

    pub async fn themes_json(_: &Vec<WmsService>, _: String, _: Option<&str>) -> String {
        unimplemented!()
    }
}

#[cfg(not(feature = "map-server"))]
pub use dummy_inventory::*;

use bytes::Bytes;
use tile_grid::{Extent, Grid};

pub struct WmsRequest {
    client: reqwest::Client,
    pub wms_url: String,
    pub layers: Vec<String>,
    pub image_type: String,
}

impl WmsRequest {
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        WmsRequest {
            client,
            wms_url: "http://localhost:8080/wms/qgs/ne".to_string(),
            layers: vec!["country".to_string()],
            image_type: "image/png;%20mode=8bit".to_string(),
        }
    }

    fn get_map_request(&self, _grid: &Grid, extent: &Extent) -> String {
        let layers = self.layers.join(",");
        let bbox = format!(
            "{},{},{},{}",
            extent.minx, extent.miny, extent.maxx, extent.maxy
        );
        format!(
            "{}?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX={}&CRS=EPSG:900913&WIDTH={}&HEIGHT={}&LAYERS={}&STYLES=&FORMAT={}",
            &self.wms_url,
            bbox,
            256, //grid.width,
            256, //grid.height,
            layers,
            &self.image_type,
        )
    }

    pub async fn get_map(&self, grid: &Grid, extent: &Extent) -> reqwest::Result<Bytes> {
        let response = self
            .client
            .get(self.get_map_request(grid, extent))
            .send()
            .await
            .unwrap();
        // if !response.status().is_success() {
        //     return Err();
        response.bytes().await
    }
}

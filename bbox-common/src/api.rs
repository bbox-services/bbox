use crate::ogcapi::*;
use utoipa::openapi::OpenApi;

#[derive(Clone)]
pub struct OgcApiInventory {
    pub landing_page_links: Vec<ApiLink>,
    pub conformance_classes: Vec<String>,
    pub collections: Vec<CoreCollection>,
}

pub trait ExtendApiDoc {
    fn extend(&mut self, api: OpenApi);
}

impl ExtendApiDoc for OpenApi {
    fn extend(&mut self, api: OpenApi) {
        self.paths.paths.extend(api.paths.paths);
        //TODO: self.components
    }
}

use crate::ogcapi::*;
use utoipa::openapi::schema::Components;
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
        let mut components1 = self.components.clone().unwrap_or(Components::new());
        let components2 = api.components.unwrap_or(Components::new());
        components1.schemas.extend(components2.schemas);
        components1
            .security_schemes
            .extend(components2.security_schemes);
        self.components = Some(components1);
    }
}

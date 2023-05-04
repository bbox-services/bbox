use actix_web::web;
use async_trait::async_trait;
use bbox_common::service::{CoreService, OgcApiService};

#[derive(Clone)]
pub struct BboxService;

#[async_trait]
impl OgcApiService for BboxService {
    async fn from_config() -> Self {
        BboxService {}
    }
    fn register_endpoints(&self, cfg: &mut web::ServiceConfig, core: &CoreService) {
        self.register(cfg, core)
    }
}

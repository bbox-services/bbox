use actix_web::web;
use async_trait::async_trait;
use bbox_core::cli::{NoArgs, NoCommands};
use bbox_core::metrics::{no_metrics, NoMetrics};
use bbox_core::service::{OgcApiService, ServiceEndpoints};
use clap::ArgMatches;

#[derive(Clone, Default)]
pub struct BboxService;

#[async_trait]
impl OgcApiService for BboxService {
    type CliCommands = NoCommands;
    type CliArgs = NoArgs;
    type Metrics = NoMetrics;

    async fn read_config(&mut self, _cli: &ArgMatches) {}
    fn metrics(&self) -> &'static Self::Metrics {
        no_metrics()
    }
}

impl ServiceEndpoints for BboxService {
    fn register_endpoints(&self, _cfg: &mut web::ServiceConfig) {}
}

use async_trait::async_trait;
use bbox_common::ogcapi::ApiLink;
use bbox_common::service::OgcApiService;

#[derive(Clone)]
pub struct BboxService;

#[async_trait]
impl OgcApiService for BboxService {
    async fn from_config() -> Self {
        BboxService {}
    }
    fn landing_page_links(&self, _api_base: &str) -> Vec<ApiLink> {
        vec![ApiLink {
            href: "/collections".to_string(),
            rel: Some("data".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("Information about the feature collections".to_string()),
            hreflang: None,
            length: None,
        }]
    }
}

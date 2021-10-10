use std::rc::Rc;

use hyper;
use super::configuration::Configuration;

pub struct APIClient {
    capabilities_api: Box<dyn crate::apis::CapabilitiesApi>,
    data_api: Box<dyn crate::apis::DataApi>,
}

impl APIClient {
    pub fn new<C: hyper::client::Connect>(configuration: Configuration<C>) -> APIClient {
        let rc = Rc::new(configuration);

        APIClient {
            capabilities_api: Box::new(crate::apis::CapabilitiesApiClient::new(rc.clone())),
            data_api: Box::new(crate::apis::DataApiClient::new(rc.clone())),
        }
    }

    pub fn capabilities_api(&self) -> &dyn crate::apis::CapabilitiesApi{
        self.capabilities_api.as_ref()
    }

    pub fn data_api(&self) -> &dyn crate::apis::DataApi{
        self.data_api.as_ref()
    }

}

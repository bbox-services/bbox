use crate::ogcapi::*;

#[derive(Clone)]
pub struct OgcApiInventory {
    pub landing_page_links: Vec<ApiLink>,
    pub conformance_classes: Vec<String>,
    pub collections: Vec<CoreCollection>,
}

/// OpenAPi doc collection
pub type OpenApiDoc = serde_yaml::Value;

pub trait OpenApiDocCollection {
    fn from_yaml(yaml: &str, prefix: &str) -> Self;
    fn extend(&mut self, yaml: &str, prefix: &str);
    fn as_yaml(&self) -> String;
    fn as_json(&self) -> String;
}

impl OpenApiDocCollection for OpenApiDoc {
    fn from_yaml(yaml: &str, _prefix: &str) -> Self {
        serde_yaml::from_str(yaml).unwrap()
    }
    fn extend(&mut self, yaml: &str, _prefix: &str) {
        let rhs_yaml: OpenApiDoc = serde_yaml::from_str(yaml).unwrap();
        if let Some(rhs_paths) = rhs_yaml.get("paths") {
            if let Some(paths) = self.get_mut("paths") {
                paths
                    .as_mapping_mut()
                    .unwrap()
                    .extend(rhs_paths.as_mapping().unwrap().clone().into_iter());
            } else {
                self.as_mapping_mut()
                    .unwrap()
                    .insert("paths".into(), rhs_paths.clone());
            }
        }
        if let Some(rhs_components) = rhs_yaml.get("components") {
            if let Some(components) = self.get_mut("components") {
                // TODO: merge 1st level children
                components
                    .as_mapping_mut()
                    .unwrap()
                    .extend(rhs_components.as_mapping().unwrap().clone().into_iter());
            } else {
                self.as_mapping_mut()
                    .unwrap()
                    .insert("components".into(), rhs_components.clone());
            }
        }
    }
    fn as_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }
    fn as_json(&self) -> String {
        let json = serde_yaml::from_value::<serde_json::Value>(self.clone()).unwrap();
        serde_json::to_string_pretty(&json).unwrap()
    }
}

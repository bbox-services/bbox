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
    fn as_json(&self) -> serde_json::Value;
}

impl OpenApiDocCollection for OpenApiDoc {
    fn from_yaml(yaml: &str, _prefix: &str) -> Self {
        serde_yaml::from_str(yaml).unwrap()
    }
    fn extend(&mut self, yaml: &str, _prefix: &str) {
        let rhs_yaml: OpenApiDoc = serde_yaml::from_str(yaml).unwrap();
        merge_level(self, &rhs_yaml, "paths");
        if let Some(rhs_components) = rhs_yaml.get("components") {
            if let Some(components) = self.get_mut("components") {
                // merge 1st level children ("parameters", "responses", "schemas")
                for (key, _val) in rhs_components.as_mapping().unwrap().iter() {
                    merge_level(components, &rhs_components, key.as_str().unwrap());
                }
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
    fn as_json(&self) -> serde_json::Value {
        serde_yaml::from_value::<serde_json::Value>(self.clone()).unwrap()
    }
}

fn merge_level(yaml: &mut serde_yaml::Value, rhs_yaml: &serde_yaml::Value, index: &str) {
    if let Some(rhs_elem) = rhs_yaml.get(index) {
        if let Some(elem) = yaml.get_mut(index) {
            elem.as_mapping_mut()
                .unwrap()
                .extend(rhs_elem.as_mapping().unwrap().clone().into_iter());
        } else {
            yaml.as_mapping_mut()
                .unwrap()
                .insert(index.into(), rhs_elem.clone());
        }
    }
}

use crate::ogcapi::*;

#[derive(Clone)]
pub struct OgcApiInventory {
    pub landing_page_links: Vec<ApiLink>,
    pub conformance_classes: Vec<String>,
    pub collections: Vec<CoreCollection>,
}

impl OgcApiInventory {
    pub fn new() -> Self {
        OgcApiInventory {
            landing_page_links: Vec::new(),
            conformance_classes: Vec::new(),
            collections: Vec::new(),
        }
    }
}

/// OpenAPi doc collection
#[derive(Clone)]
pub struct OpenApiDoc(serde_yaml::Value);

impl OpenApiDoc {
    pub fn new() -> Self {
        Self::from_yaml("{}", "")
    }
    pub fn from_yaml(yaml: &str, _prefix: &str) -> Self {
        OpenApiDoc(serde_yaml::from_str(yaml).unwrap())
    }
    pub fn extend(&mut self, yaml: &str, _prefix: &str) {
        let rhs_yaml = serde_yaml::from_str(yaml).unwrap();
        merge_level(&mut self.0, &rhs_yaml, "paths");
        if let Some(rhs_components) = rhs_yaml.get("components") {
            if let Some(components) = self.0.get_mut("components") {
                // merge 1st level children ("parameters", "responses", "schemas")
                for (key, _val) in rhs_components.as_mapping().unwrap().iter() {
                    merge_level(components, &rhs_components, key.as_str().unwrap());
                }
            } else {
                self.0
                    .as_mapping_mut()
                    .unwrap()
                    .insert("components".into(), rhs_components.clone());
            }
        }
    }
    pub fn as_yaml(&self) -> String {
        serde_yaml::to_string(&self.0).unwrap()
    }
    pub fn as_json(&self) -> serde_json::Value {
        serde_yaml::from_value::<serde_json::Value>(self.0.clone()).unwrap()
    }
    pub fn nop(&self) {}
}

fn merge_level(yaml: &mut serde_yaml::Value, rhs_yaml: &serde_yaml::Value, key: &str) {
    if let Some(rhs_elem) = rhs_yaml.get(key) {
        if let Some(elem) = yaml.get_mut(key) {
            elem.as_mapping_mut()
                .unwrap()
                .extend(rhs_elem.as_mapping().unwrap().clone().into_iter());
        } else {
            yaml.as_mapping_mut()
                .unwrap()
                .insert(key.into(), rhs_elem.clone());
        }
    }
}

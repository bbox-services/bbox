use crate::ogcapi::*;

#[derive(Clone, Default)]
pub struct OgcApiInventory {
    pub landing_page_links: Vec<ApiLink>,
    pub conformance_classes: Vec<String>,
    pub collections: Vec<CoreCollection>,
}

/// OpenAPi doc collection
#[derive(Default, Clone)]
pub struct OpenApiDoc(serde_yaml::Value);

impl OpenApiDoc {
    pub fn new() -> Self {
        Self::from_yaml("{}", "")
    }
    pub fn from_yaml(yaml: &str, _prefix: &str) -> Self {
        OpenApiDoc(serde_yaml::from_str(yaml).unwrap())
    }
    pub fn is_empty(&self) -> bool {
        self.0 == Self::new().0
    }
    /// Merge `paths` and `components` of new yaml into exisiting yaml
    pub fn extend(&mut self, yaml: &str, _prefix: &str) {
        let rhs_yaml = serde_yaml::from_str(yaml).unwrap();
        merge_level(&mut self.0, &rhs_yaml, "paths");
        if let Some(rhs_components) = rhs_yaml.get("components") {
            if let Some(components) = self.0.get_mut("components") {
                // merge 1st level children ("parameters", "responses", "schemas")
                for (key, _val) in rhs_components.as_mapping().unwrap().iter() {
                    merge_level(components, rhs_components, key.as_str().unwrap());
                }
            } else {
                self.0
                    .as_mapping_mut()
                    .unwrap()
                    .insert("components".into(), rhs_components.clone());
            }
        }
    }
    /// Set url of first server entry
    pub fn set_server_url(&mut self, url: &str) {
        if let Some(servers) = self.0.get_mut("servers") {
            if let Some(server) = servers.get_mut(0) {
                if let Some(server) = server.as_mapping_mut() {
                    server[&"url".to_string().into()] = url.to_string().into();
                }
            }
        }
    }
    pub fn as_yaml(&self, public_server_url: &str) -> String {
        let mut doc = self.clone();
        doc.set_server_url(public_server_url);
        serde_yaml::to_string(&doc.0).unwrap()
    }
    pub fn as_json(&self, public_server_url: &str) -> serde_json::Value {
        let mut doc = self.clone();
        doc.set_server_url(public_server_url);
        serde_yaml::from_value::<serde_json::Value>(doc.0).unwrap()
    }
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

#[cfg(test)]
mod test {
    use super::*;

    const YAML_BASE: &str = r##"---
openapi: 3.0.2
info:
  title: BBOX OGC API
servers:
  - url: "http://bbox:8080/"
    description: Production server
paths:
  /conformance:
    get:
      operationId: getConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
components:
  schemas:
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
"##;

    #[test]
    fn yaml_empty() {
        let doc = OpenApiDoc::new();
        assert!(doc.is_empty());
    }

    #[test]
    fn yaml_extend_headers() {
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), YAML_BASE);

        let yaml = r##"---
openapi: 3.0.0
info:
  title: New Title
"##;
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), YAML_BASE);
    }

    #[test]
    fn yaml_add_path() {
        let yaml = r##"---
paths:
  /newpath: ""
"##;
        let yamlout = r##"---
openapi: 3.0.2
info:
  title: BBOX OGC API
servers:
  - url: "http://bbox:8080/"
    description: Production server
paths:
  /conformance:
    get:
      operationId: getConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
  /newpath: ""
components:
  schemas:
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
"##;
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), yamlout);
    }

    #[test]
    fn yaml_update_path() {
        let yaml = r##"---
paths:
  /conformance:
    get:
      operationId: getConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
"##;
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), YAML_BASE);
    }

    #[test]
    fn yaml_change_path() {
        let yaml = r##"---
paths:
  /conformance:
    post:
      operationId: postConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
"##;
        let yamlout = r##"---
openapi: 3.0.2
info:
  title: BBOX OGC API
servers:
  - url: "http://bbox:8080/"
    description: Production server
paths:
  /conformance:
    post:
      operationId: postConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
components:
  schemas:
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
"##;
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), yamlout);
    }

    #[test]
    fn yaml_add_component() {
        let yaml = r##"---
components:
  schemas:
    link:
      type: object
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
"##;
        let yamlout = r##"---
openapi: 3.0.2
info:
  title: BBOX OGC API
servers:
  - url: "http://bbox:8080/"
    description: Production server
paths:
  /conformance:
    get:
      operationId: getConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
components:
  schemas:
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
    link:
      type: object
"##;
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), yamlout);
    }

    #[test]
    fn yaml_add_new_component() {
        let yaml = r##"---
components:
  responses:
    NotFound:
      description: The requested resource does not exist.
"##;
        let yamlout = r##"---
openapi: 3.0.2
info:
  title: BBOX OGC API
servers:
  - url: "http://bbox:8080/"
    description: Production server
paths:
  /conformance:
    get:
      operationId: getConformance
      responses:
        "333":
          $ref: "#/components/schemas/confClasses"
components:
  schemas:
    confClasses:
      type: object
      required:
        - conformsTo
      properties:
        conformsTo:
          type: array
          items:
            type: string
  responses:
    NotFound:
      description: The requested resource does not exist.
"##;
        let mut doc = OpenApiDoc::from_yaml(YAML_BASE, "");
        doc.extend(yaml, "");
        assert_eq!(doc.as_yaml("http://bbox:8080/"), yamlout);
    }
}

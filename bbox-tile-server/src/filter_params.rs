use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FilterParams {
    pub datetime: Option<String>,
    pub filters: HashMap<String, String>,
}

#[derive(Debug)]
pub enum TemporalType {
    DateTime(chrono::DateTime<chrono::FixedOffset>),
    Open,
}

impl FilterParams {
    pub fn as_args(&self) -> String {
        let mut args = vec![
            Some("".to_string()),
            self.datetime.as_ref().map(|v| format!("datetime={v}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("&");

        for (key, val) in &self.filters {
            args.push_str(&format!("&{key}={val}"))
        }
        if !args.is_empty() {
            // replace & with ?
            args.replace_range(0..1, "?");
        }
        args
    }
    pub fn temporal(&self) -> Result<Option<Vec<TemporalType>>, Box<dyn std::error::Error>> {
        if let Some(dt) = &self.datetime {
            let parts: Vec<&str> = dt.split('/').collect();
            let mut parsed_parts = vec![];
            for part in &parts {
                match *part {
                    ".." => parsed_parts.push(TemporalType::Open),
                    p => {
                        parsed_parts.push(TemporalType::DateTime(
                            chrono::DateTime::parse_from_rfc3339(p)?,
                        ));
                    }
                }
            }
            return Ok(Some(parsed_parts));
        }
        Ok(None)
    }
    pub fn other_params(&self) -> Result<&HashMap<String, String>, Box<dyn std::error::Error>> {
        Ok(&self.filters)
    }
}

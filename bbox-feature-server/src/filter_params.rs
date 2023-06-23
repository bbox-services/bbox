use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(deny_unknown_fields)] // http://docs.opengeospatial.org/DRAFTS/17-069r5.html#query_parameters
pub struct FilterParams {
    // Pagination
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    // Filter
    pub bbox: Option<String>,
    // TODO: interval
}

impl FilterParams {
    pub fn limit_or_default(&self) -> u32 {
        self.limit.unwrap_or(50)
    }
    pub fn with_offset(&self, offset: u32) -> FilterParams {
        let mut params = self.clone();
        params.offset = Some(offset);
        params
    }
    pub fn prev(&self) -> Option<FilterParams> {
        let offset = self.offset.unwrap_or(0);
        if offset > 0 {
            let prev = offset.saturating_sub(self.limit_or_default());
            Some(self.with_offset(prev))
        } else {
            None
        }
    }
    pub fn next(&self, max: u64) -> Option<FilterParams> {
        let offset = self.offset.unwrap_or(0);
        let next = offset.saturating_add(self.limit_or_default());
        if (next as u64) < max {
            Some(self.with_offset(next))
        } else {
            None
        }
    }
    pub fn as_args(&self) -> String {
        let mut args = vec![
            Some("".to_string()),
            self.limit.map(|v| format!("limit={v}")),
            self.offset.map(|v| format!("offset={v}")),
            self.bbox.as_ref().map(|v| format!("bbox={v}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join("&");
        if !args.is_empty() {
            // replace & with ?
            args.replace_range(0..1, "?");
        }
        args
    }
    pub fn bbox(&self) -> Result<Option<Vec<f64>>, std::num::ParseFloatError> {
        if let Some(bboxstr) = &self.bbox {
            let bbox: Vec<f64> = bboxstr
                .split(',')
                .map(|v| v.parse())
                .collect::<Result<Vec<_>, _>>()?;
            if bbox.len() == 4 || bbox.len() == 6 {
                return Ok(Some(bbox));
            }
            // TODO: else return Err
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_to_args() {
        let filter = FilterParams {
            limit: Some(10),
            offset: Some(20),
            bbox: Some("1.0,2.2,3.33,4.444".to_string()),
        };
        assert_eq!(
            filter.as_args(),
            "?limit=10&offset=20&bbox=1.0,2.2,3.33,4.444"
        );

        let filter = FilterParams {
            limit: None,
            offset: Some(20),
            bbox: None,
        };
        assert_eq!(filter.as_args(), "?offset=20");

        let filter = FilterParams {
            limit: None,
            offset: None,
            bbox: None,
        };
        assert_eq!(filter.as_args(), "");
    }

    #[test]
    fn prev_next() {
        let filter = FilterParams {
            limit: Some(10),
            offset: Some(20),
            bbox: None,
        };
        assert_eq!(filter.prev().unwrap().offset, Some(10));
        assert_eq!(filter.next(35).unwrap().offset, Some(30));
        assert!(filter.next(20).is_none());
        assert!(filter.next(19).is_none());

        let filter = FilterParams {
            limit: Some(10),
            offset: Some(10),
            bbox: None,
        };
        assert_eq!(filter.prev().unwrap().offset, Some(0));
        assert_eq!(filter.next(35).unwrap().offset, Some(20));

        let filter = FilterParams {
            limit: Some(10),
            offset: None,
            bbox: None,
        };
        assert!(filter.prev().is_none());
        assert_eq!(filter.next(35).unwrap().offset, Some(10));
    }

    #[test]
    fn bbox_parse() {
        assert_eq!(
            FilterParams {
                limit: None,
                offset: None,
                bbox: Some("1.0,2.2,3.33,4.444".to_string()),
            }
            .bbox()
            .unwrap(),
            Some(vec![1.0, 2.2, 3.33, 4.444])
        );

        assert_eq!(
            FilterParams {
                limit: None,
                offset: None,
                bbox: Some("1.0,2.2,3.33,4.444,5,6".to_string()),
            }
            .bbox()
            .unwrap(),
            Some(vec![1.0, 2.2, 3.33, 4.444, 5.0, 6.0])
        );

        assert!(FilterParams {
            limit: None,
            offset: None,
            bbox: Some("1.0, 2.2, 3.33, 4.444".to_string()),
        }
        .bbox()
        .is_err());

        assert_eq!(
            FilterParams {
                limit: None,
                offset: None,
                bbox: Some("1,2,3".to_string()),
            }
            .bbox()
            .unwrap(),
            None // should be Err
        );
    }
}

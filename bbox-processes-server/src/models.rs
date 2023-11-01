// Generated with OpenAPI Generator, with manual patches.
#![allow(unused_qualifications)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(clippy::all)]

use crate::models;
// #[cfg(any(feature = "client", feature = "server"))]
// use crate::header;

// Workaround for missing structs
#[allow(non_camel_case_types)]
type schemaYaml = String;
#[allow(non_camel_case_types)]
type referenceYaml = String;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdditionalParameter {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "value")]
    pub value: Vec<swagger::OneOf5<String, f64, i32, Vec<serde_json::Value>, serde_json::Value>>,
}

impl AdditionalParameter {
    pub fn new(
        name: String,
        value: Vec<swagger::OneOf5<String, f64, i32, Vec<serde_json::Value>, serde_json::Value>>,
    ) -> AdditionalParameter {
        AdditionalParameter {
            name: name,
            value: value,
        }
    }
}

/// Converts the AdditionalParameter value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AdditionalParameter {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("name".to_string());
        params.push(self.name.to_string());

        // Skipping value in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AdditionalParameter value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AdditionalParameter {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub name: Vec<String>,
            pub value: Vec<
                Vec<swagger::OneOf5<String, f64, i32, Vec<serde_json::Value>, serde_json::Value>>,
            >,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AdditionalParameter".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "name" => intermediate_rep.name.push(
                        <String as std::str::FromStr>::from_str(val).map_err(|x| format!("{x}"))?,
                    ),
                    "value" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in AdditionalParameter"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AdditionalParameter".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AdditionalParameter {
            name: intermediate_rep
                .name
                .into_iter()
                .next()
                .ok_or("name missing in AdditionalParameter".to_string())?,
            value: intermediate_rep
                .value
                .into_iter()
                .next()
                .ok_or("value missing in AdditionalParameter".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AdditionalParameter> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AdditionalParameter>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AdditionalParameter>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AdditionalParameter - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AdditionalParameter>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AdditionalParameter as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AdditionalParameter - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConfClasses {
    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}

impl ConfClasses {
    pub fn new(conforms_to: Vec<String>) -> ConfClasses {
        ConfClasses {
            conforms_to: conforms_to,
        }
    }
}

/// Converts the ConfClasses value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ConfClasses {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("conformsTo".to_string());
        params.push(
            self.conforms_to
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
                .to_string(),
        );

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ConfClasses value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ConfClasses {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub conforms_to: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ConfClasses".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "conformsTo" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ConfClasses"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ConfClasses".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ConfClasses {
            conforms_to: intermediate_rep
                .conforms_to
                .into_iter()
                .next()
                .ok_or("conformsTo missing in ConfClasses".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ConfClasses> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ConfClasses>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ConfClasses>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ConfClasses - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ConfClasses> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ConfClasses as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ConfClasses - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DescriptionType {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "keywords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<models::Metadata>>,

    #[serde(rename = "additionalParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<Metadata>,
}

impl DescriptionType {
    pub fn new() -> DescriptionType {
        DescriptionType {
            title: None,
            description: None,
            keywords: None,
            metadata: None,
            additional_parameters: None,
        }
    }
}

/// Converts the DescriptionType value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for DescriptionType {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref keywords) = self.keywords {
            params.push("keywords".to_string());
            params.push(
                keywords
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping metadata in query parameter serialization

        // Skipping additionalParameters in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a DescriptionType value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for DescriptionType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub keywords: Vec<Vec<String>>,
            pub metadata: Vec<Vec<models::Metadata>>,
            pub additional_parameters: Vec<Metadata>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing DescriptionType".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "keywords" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in DescriptionType"
                                .to_string(),
                        )
                    }
                    "metadata" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in DescriptionType"
                                .to_string(),
                        )
                    }
                    "additionalParameters" => intermediate_rep.additional_parameters.push(
                        <Metadata as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing DescriptionType".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(DescriptionType {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            keywords: intermediate_rep.keywords.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
            additional_parameters: intermediate_rep.additional_parameters.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<DescriptionType> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<DescriptionType>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<DescriptionType>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for DescriptionType - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<DescriptionType>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <DescriptionType as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into DescriptionType - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// JSON schema for exceptions based on RFC 7807
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Exception {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<isize>,

    #[serde(rename = "detail")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(rename = "instance")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl Exception {
    pub fn new(type_: String) -> Exception {
        Exception {
            type_: type_,
            title: None,
            status: None,
            detail: None,
            instance: None,
        }
    }
}

/// Converts the Exception value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Exception {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref status) = self.status {
            params.push("status".to_string());
            params.push(status.to_string());
        }

        if let Some(ref detail) = self.detail {
            params.push("detail".to_string());
            params.push(detail.to_string());
        }

        if let Some(ref instance) = self.instance {
            params.push("instance".to_string());
            params.push(instance.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Exception value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Exception {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub title: Vec<String>,
            pub status: Vec<isize>,
            pub detail: Vec<String>,
            pub instance: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Exception".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "status" => intermediate_rep.status.push(
                        <isize as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "detail" => intermediate_rep.detail.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "instance" => intermediate_rep.instance.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Exception".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Exception {
            type_: intermediate_rep
                .type_
                .into_iter()
                .next()
                .ok_or("type missing in Exception".to_string())?,
            title: intermediate_rep.title.into_iter().next(),
            status: intermediate_rep.status.into_iter().next(),
            detail: intermediate_rep.detail.into_iter().next(),
            instance: intermediate_rep.instance.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Exception> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Exception>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Exception>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Exception - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Exception> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Exception as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Exception - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InputDescription {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "keywords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<models::Metadata>>,

    #[serde(rename = "additionalParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<Metadata>,

    #[serde(rename = "minOccurs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_occurs: Option<isize>,

    #[serde(rename = "maxOccurs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_occurs: Option<swagger::OneOf2<i32, String>>,

    #[serde(rename = "schema")]
    pub schema: models::Schema,
}

impl InputDescription {
    pub fn new(schema: models::Schema) -> InputDescription {
        InputDescription {
            title: None,
            description: None,
            keywords: None,
            metadata: None,
            additional_parameters: None,
            min_occurs: Some(1),
            max_occurs: None,
            schema: schema,
        }
    }
}

/// Converts the InputDescription value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InputDescription {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref keywords) = self.keywords {
            params.push("keywords".to_string());
            params.push(
                keywords
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping metadata in query parameter serialization

        // Skipping additionalParameters in query parameter serialization

        if let Some(ref min_occurs) = self.min_occurs {
            params.push("minOccurs".to_string());
            params.push(min_occurs.to_string());
        }

        // Skipping maxOccurs in query parameter serialization

        // Skipping schema in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InputDescription value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InputDescription {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub keywords: Vec<Vec<String>>,
            pub metadata: Vec<Vec<models::Metadata>>,
            pub additional_parameters: Vec<Metadata>,
            pub min_occurs: Vec<isize>,
            pub max_occurs: Vec<swagger::OneOf2<i32, String>>,
            pub schema: Vec<models::Schema>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InputDescription".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "keywords" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in InputDescription"
                            .to_string(),
                    ),
                    "metadata" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in InputDescription"
                            .to_string(),
                    ),
                    "additionalParameters" => intermediate_rep.additional_parameters.push(
                        <Metadata as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "minOccurs" => intermediate_rep.min_occurs.push(
                        <isize as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "maxOccurs" => intermediate_rep.max_occurs.push(
                        <swagger::OneOf2<i32, String> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "schema" => intermediate_rep.schema.push(
                        <models::Schema as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InputDescription".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InputDescription {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            keywords: intermediate_rep.keywords.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
            additional_parameters: intermediate_rep.additional_parameters.into_iter().next(),
            min_occurs: intermediate_rep.min_occurs.into_iter().next(),
            max_occurs: intermediate_rep.max_occurs.into_iter().next(),
            schema: intermediate_rep
                .schema
                .into_iter()
                .next()
                .ok_or("schema missing in InputDescription".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InputDescription> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<InputDescription>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InputDescription>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InputDescription - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<InputDescription>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InputDescription as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InputDescription - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InputDescriptionAllOf {
    #[serde(rename = "minOccurs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_occurs: Option<isize>,

    #[serde(rename = "maxOccurs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_occurs: Option<swagger::OneOf2<i32, String>>,

    #[serde(rename = "schema")]
    pub schema: models::Schema,
}

impl InputDescriptionAllOf {
    pub fn new(schema: models::Schema) -> InputDescriptionAllOf {
        InputDescriptionAllOf {
            min_occurs: Some(1),
            max_occurs: None,
            schema: schema,
        }
    }
}

/// Converts the InputDescriptionAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InputDescriptionAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref min_occurs) = self.min_occurs {
            params.push("minOccurs".to_string());
            params.push(min_occurs.to_string());
        }

        // Skipping maxOccurs in query parameter serialization

        // Skipping schema in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InputDescriptionAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InputDescriptionAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub min_occurs: Vec<isize>,
            pub max_occurs: Vec<swagger::OneOf2<i32, String>>,
            pub schema: Vec<models::Schema>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing InputDescriptionAllOf".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "minOccurs" => intermediate_rep.min_occurs.push(
                        <isize as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "maxOccurs" => intermediate_rep.max_occurs.push(
                        <swagger::OneOf2<i32, String> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "schema" => intermediate_rep.schema.push(
                        <models::Schema as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing InputDescriptionAllOf".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InputDescriptionAllOf {
            min_occurs: intermediate_rep.min_occurs.into_iter().next(),
            max_occurs: intermediate_rep.max_occurs.into_iter().next(),
            schema: intermediate_rep
                .schema
                .into_iter()
                .next()
                .ok_or("schema missing in InputDescriptionAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InputDescriptionAllOf> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<InputDescriptionAllOf>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<InputDescriptionAllOf>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for InputDescriptionAllOf - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<InputDescriptionAllOf>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <InputDescriptionAllOf as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into InputDescriptionAllOf - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum JobControlOptions {
    #[serde(rename = "sync-execute")]
    SYNC_EXECUTE,
    #[serde(rename = "async-execute")]
    ASYNC_EXECUTE,
    #[serde(rename = "dismiss")]
    DISMISS,
}

impl std::fmt::Display for JobControlOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            JobControlOptions::SYNC_EXECUTE => write!(f, "{}", "sync-execute"),
            JobControlOptions::ASYNC_EXECUTE => write!(f, "{}", "async-execute"),
            JobControlOptions::DISMISS => write!(f, "{}", "dismiss"),
        }
    }
}

impl std::str::FromStr for JobControlOptions {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "sync-execute" => std::result::Result::Ok(JobControlOptions::SYNC_EXECUTE),
            "async-execute" => std::result::Result::Ok(JobControlOptions::ASYNC_EXECUTE),
            "dismiss" => std::result::Result::Ok(JobControlOptions::DISMISS),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JobList {
    #[serde(rename = "jobs")]
    pub jobs: Vec<models::StatusInfo>,

    #[serde(rename = "links")]
    pub links: Vec<models::Link>,
}

impl JobList {
    pub fn new(jobs: Vec<models::StatusInfo>, links: Vec<models::Link>) -> JobList {
        JobList {
            jobs: jobs,
            links: links,
        }
    }
}

/// Converts the JobList value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for JobList {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping jobs in query parameter serialization

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a JobList value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for JobList {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub jobs: Vec<Vec<models::StatusInfo>>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing JobList".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "jobs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in JobList"
                                .to_string(),
                        )
                    }
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in JobList"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing JobList".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(JobList {
            jobs: intermediate_rep
                .jobs
                .into_iter()
                .next()
                .ok_or("jobs missing in JobList".to_string())?,
            links: intermediate_rep
                .links
                .into_iter()
                .next()
                .ok_or("links missing in JobList".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<JobList> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<JobList>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<JobList>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for JobList - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<JobList> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <JobList as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into JobList - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LandingPage {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "links")]
    pub links: Vec<models::Link>,
}

impl LandingPage {
    pub fn new(links: Vec<models::Link>) -> LandingPage {
        LandingPage {
            title: None,
            description: None,
            links: links,
        }
    }
}

/// Converts the LandingPage value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for LandingPage {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a LandingPage value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for LandingPage {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing LandingPage".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in LandingPage"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing LandingPage".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(LandingPage {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            links: intermediate_rep
                .links
                .into_iter()
                .next()
                .ok_or("links missing in LandingPage".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<LandingPage> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<LandingPage>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<LandingPage>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for LandingPage - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<LandingPage> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <LandingPage as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into LandingPage - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Link {
    #[serde(rename = "href")]
    pub href: String,

    #[serde(rename = "rel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,

    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    #[serde(rename = "hreflang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Link {
    pub fn new(href: String) -> Link {
        Link {
            href: href,
            rel: None,
            type_: None,
            hreflang: None,
            title: None,
        }
    }
}

/// Converts the Link value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Link {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("href".to_string());
        params.push(self.href.to_string());

        if let Some(ref rel) = self.rel {
            params.push("rel".to_string());
            params.push(rel.to_string());
        }

        if let Some(ref type_) = self.type_ {
            params.push("type".to_string());
            params.push(type_.to_string());
        }

        if let Some(ref hreflang) = self.hreflang {
            params.push("hreflang".to_string());
            params.push(hreflang.to_string());
        }

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Link value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Link {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub href: Vec<String>,
            pub rel: Vec<String>,
            pub type_: Vec<String>,
            pub hreflang: Vec<String>,
            pub title: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err("Missing value while parsing Link".to_string())
                }
            };

            if let Some(key) = key_result {
                match key {
                    "href" => intermediate_rep.href.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "rel" => intermediate_rep.rel.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "type" => intermediate_rep.type_.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "hreflang" => intermediate_rep.hreflang.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Link".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Link {
            href: intermediate_rep
                .href
                .into_iter()
                .next()
                .ok_or("href missing in Link".to_string())?,
            rel: intermediate_rep.rel.into_iter().next(),
            type_: intermediate_rep.type_.into_iter().next(),
            hreflang: intermediate_rep.hreflang.into_iter().next(),
            title: intermediate_rep.title.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Link> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Link>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Link>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Link - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Link> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => match <Link as std::str::FromStr>::from_str(value) {
                std::result::Result::Ok(value) => {
                    std::result::Result::Ok(header::IntoHeaderValue(value))
                }
                std::result::Result::Err(err) => std::result::Result::Err(format!(
                    "Unable to convert header value '{}' into Link - {}",
                    value, err
                )),
            },
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(rename = "href")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            title: None,
            role: None,
            href: None,
        }
    }
}

/// Converts the Metadata value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Metadata {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref role) = self.role {
            params.push("role".to_string());
            params.push(role.to_string());
        }

        if let Some(ref href) = self.href {
            params.push("href".to_string());
            params.push(href.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Metadata value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Metadata {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub role: Vec<String>,
            pub href: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Metadata".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "role" => intermediate_rep.role.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "href" => intermediate_rep.href.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Metadata".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Metadata {
            title: intermediate_rep.title.into_iter().next(),
            role: intermediate_rep.role.into_iter().next(),
            href: intermediate_rep.href.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Metadata> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Metadata>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Metadata>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Metadata - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Metadata> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Metadata as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Metadata - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputDescription {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "keywords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<models::Metadata>>,

    #[serde(rename = "additionalParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<Metadata>,

    #[serde(rename = "schema")]
    pub schema: models::Schema,
}

impl OutputDescription {
    pub fn new(schema: models::Schema) -> OutputDescription {
        OutputDescription {
            title: None,
            description: None,
            keywords: None,
            metadata: None,
            additional_parameters: None,
            schema: schema,
        }
    }
}

/// Converts the OutputDescription value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for OutputDescription {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref keywords) = self.keywords {
            params.push("keywords".to_string());
            params.push(
                keywords
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping metadata in query parameter serialization

        // Skipping additionalParameters in query parameter serialization

        // Skipping schema in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a OutputDescription value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for OutputDescription {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub keywords: Vec<Vec<String>>,
            pub metadata: Vec<Vec<models::Metadata>>,
            pub additional_parameters: Vec<Metadata>,
            pub schema: Vec<models::Schema>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing OutputDescription".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "keywords" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in OutputDescription"
                            .to_string(),
                    ),
                    "metadata" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in OutputDescription"
                            .to_string(),
                    ),
                    "additionalParameters" => intermediate_rep.additional_parameters.push(
                        <Metadata as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "schema" => intermediate_rep.schema.push(
                        <models::Schema as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing OutputDescription".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(OutputDescription {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            keywords: intermediate_rep.keywords.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
            additional_parameters: intermediate_rep.additional_parameters.into_iter().next(),
            schema: intermediate_rep
                .schema
                .into_iter()
                .next()
                .ok_or("schema missing in OutputDescription".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<OutputDescription> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<OutputDescription>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<OutputDescription>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for OutputDescription - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<OutputDescription>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <OutputDescription as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into OutputDescription - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutputDescriptionAllOf {
    #[serde(rename = "schema")]
    pub schema: models::Schema,
}

impl OutputDescriptionAllOf {
    pub fn new(schema: models::Schema) -> OutputDescriptionAllOf {
        OutputDescriptionAllOf { schema: schema }
    }
}

/// Converts the OutputDescriptionAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for OutputDescriptionAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping schema in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a OutputDescriptionAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for OutputDescriptionAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub schema: Vec<models::Schema>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing OutputDescriptionAllOf".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "schema" => intermediate_rep.schema.push(
                        <models::Schema as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing OutputDescriptionAllOf".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(OutputDescriptionAllOf {
            schema: intermediate_rep
                .schema
                .into_iter()
                .next()
                .ok_or("schema missing in OutputDescriptionAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<OutputDescriptionAllOf> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<OutputDescriptionAllOf>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<OutputDescriptionAllOf>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for OutputDescriptionAllOf - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<OutputDescriptionAllOf>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <OutputDescriptionAllOf as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into OutputDescriptionAllOf - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Process {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "keywords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<models::Metadata>>,

    #[serde(rename = "additionalParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<Metadata>,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "jobControlOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_control_options: Option<Vec<models::JobControlOptions>>,

    #[serde(rename = "outputTransmission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_transmission: Option<Vec<models::TransmissionMode>>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<models::Link>>,

    #[serde(rename = "inputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<std::collections::HashMap<String, models::InputDescription>>,

    #[serde(rename = "outputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<std::collections::HashMap<String, models::OutputDescription>>,
}

impl Process {
    pub fn new(id: String, version: String) -> Process {
        Process {
            title: None,
            description: None,
            keywords: None,
            metadata: None,
            additional_parameters: None,
            id: id,
            version: version,
            job_control_options: None,
            output_transmission: None,
            links: None,
            inputs: None,
            outputs: None,
        }
    }
}

/// Converts the Process value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Process {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref keywords) = self.keywords {
            params.push("keywords".to_string());
            params.push(
                keywords
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping metadata in query parameter serialization

        // Skipping additionalParameters in query parameter serialization

        params.push("id".to_string());
        params.push(self.id.to_string());

        params.push("version".to_string());
        params.push(self.version.to_string());

        // Skipping jobControlOptions in query parameter serialization

        // Skipping outputTransmission in query parameter serialization

        // Skipping links in query parameter serialization

        // Skipping inputs in query parameter serialization
        // Skipping inputs in query parameter serialization

        // Skipping outputs in query parameter serialization
        // Skipping outputs in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Process value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Process {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub keywords: Vec<Vec<String>>,
            pub metadata: Vec<Vec<models::Metadata>>,
            pub additional_parameters: Vec<Metadata>,
            pub id: Vec<String>,
            pub version: Vec<String>,
            pub job_control_options: Vec<Vec<models::JobControlOptions>>,
            pub output_transmission: Vec<Vec<models::TransmissionMode>>,
            pub links: Vec<Vec<models::Link>>,
            pub inputs: Vec<std::collections::HashMap<String, models::InputDescription>>,
            pub outputs: Vec<std::collections::HashMap<String, models::OutputDescription>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Process".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "keywords" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "metadata" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "additionalParameters" => intermediate_rep.additional_parameters.push(
                        <Metadata as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "id" => intermediate_rep.id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "jobControlOptions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "outputTransmission" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "inputs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    "outputs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Process"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Process".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Process {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            keywords: intermediate_rep.keywords.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
            additional_parameters: intermediate_rep.additional_parameters.into_iter().next(),
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or("id missing in Process".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or("version missing in Process".to_string())?,
            job_control_options: intermediate_rep.job_control_options.into_iter().next(),
            output_transmission: intermediate_rep.output_transmission.into_iter().next(),
            links: intermediate_rep.links.into_iter().next(),
            inputs: intermediate_rep.inputs.into_iter().next(),
            outputs: intermediate_rep.outputs.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Process> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Process>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Process>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Process - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Process> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Process as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Process - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProcessAllOf {
    #[serde(rename = "inputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<std::collections::HashMap<String, models::InputDescription>>,

    #[serde(rename = "outputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<std::collections::HashMap<String, models::OutputDescription>>,
}

impl ProcessAllOf {
    pub fn new() -> ProcessAllOf {
        ProcessAllOf {
            inputs: None,
            outputs: None,
        }
    }
}

/// Converts the ProcessAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ProcessAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping inputs in query parameter serialization
        // Skipping inputs in query parameter serialization

        // Skipping outputs in query parameter serialization
        // Skipping outputs in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProcessAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProcessAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub inputs: Vec<std::collections::HashMap<String, models::InputDescription>>,
            pub outputs: Vec<std::collections::HashMap<String, models::OutputDescription>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ProcessAllOf".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "inputs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessAllOf"
                                .to_string(),
                        )
                    }
                    "outputs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessAllOf"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ProcessAllOf".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProcessAllOf {
            inputs: intermediate_rep.inputs.into_iter().next(),
            outputs: intermediate_rep.outputs.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProcessAllOf> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ProcessAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ProcessAllOf>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ProcessAllOf - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ProcessAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ProcessAllOf as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ProcessAllOf - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Information about the available processes
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProcessList {
    #[serde(rename = "processes")]
    pub processes: Vec<ProcessSummary>,

    #[serde(rename = "links")]
    pub links: Vec<Link>,
}

impl ProcessList {
    pub fn new(processes: Vec<models::ProcessSummary>, links: Vec<models::Link>) -> ProcessList {
        ProcessList {
            processes: processes,
            links: links,
        }
    }
}

/// Converts the ProcessList value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ProcessList {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping processes in query parameter serialization

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProcessList value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProcessList {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub processes: Vec<Vec<models::ProcessSummary>>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ProcessList".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "processes" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessList"
                                .to_string(),
                        )
                    }
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessList"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ProcessList".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProcessList {
            processes: intermediate_rep
                .processes
                .into_iter()
                .next()
                .ok_or("processes missing in ProcessList".to_string())?,
            links: intermediate_rep
                .links
                .into_iter()
                .next()
                .ok_or("links missing in ProcessList".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProcessList> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ProcessList>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ProcessList>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ProcessList - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ProcessList> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ProcessList as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ProcessList - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProcessSummary {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "keywords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    #[serde(rename = "metadata")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Vec<models::Metadata>>,

    #[serde(rename = "additionalParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_parameters: Option<Metadata>,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "jobControlOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_control_options: Option<Vec<models::JobControlOptions>>,

    #[serde(rename = "outputTransmission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_transmission: Option<Vec<models::TransmissionMode>>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<models::Link>>,
}

impl ProcessSummary {
    pub fn new(id: String, version: String) -> ProcessSummary {
        ProcessSummary {
            title: None,
            description: None,
            keywords: None,
            metadata: None,
            additional_parameters: None,
            id: id,
            version: version,
            job_control_options: None,
            output_transmission: None,
            links: None,
        }
    }
}

/// Converts the ProcessSummary value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ProcessSummary {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref keywords) = self.keywords {
            params.push("keywords".to_string());
            params.push(
                keywords
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping metadata in query parameter serialization

        // Skipping additionalParameters in query parameter serialization

        params.push("id".to_string());
        params.push(self.id.to_string());

        params.push("version".to_string());
        params.push(self.version.to_string());

        // Skipping jobControlOptions in query parameter serialization

        // Skipping outputTransmission in query parameter serialization

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProcessSummary value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProcessSummary {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub description: Vec<String>,
            pub keywords: Vec<Vec<String>>,
            pub metadata: Vec<Vec<models::Metadata>>,
            pub additional_parameters: Vec<Metadata>,
            pub id: Vec<String>,
            pub version: Vec<String>,
            pub job_control_options: Vec<Vec<models::JobControlOptions>>,
            pub output_transmission: Vec<Vec<models::TransmissionMode>>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ProcessSummary".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep.description.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "keywords" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessSummary"
                                .to_string(),
                        )
                    }
                    "metadata" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessSummary"
                                .to_string(),
                        )
                    }
                    "additionalParameters" => intermediate_rep.additional_parameters.push(
                        <Metadata as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "id" => intermediate_rep.id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "jobControlOptions" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessSummary"
                                .to_string(),
                        )
                    }
                    "outputTransmission" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessSummary"
                                .to_string(),
                        )
                    }
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ProcessSummary"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ProcessSummary".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProcessSummary {
            title: intermediate_rep.title.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            keywords: intermediate_rep.keywords.into_iter().next(),
            metadata: intermediate_rep.metadata.into_iter().next(),
            additional_parameters: intermediate_rep.additional_parameters.into_iter().next(),
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or("id missing in ProcessSummary".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or("version missing in ProcessSummary".to_string())?,
            job_control_options: intermediate_rep.job_control_options.into_iter().next(),
            output_transmission: intermediate_rep.output_transmission.into_iter().next(),
            links: intermediate_rep.links.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProcessSummary> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ProcessSummary>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ProcessSummary>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ProcessSummary - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ProcessSummary> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ProcessSummary as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ProcessSummary - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProcessSummaryAllOf {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "jobControlOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_control_options: Option<Vec<models::JobControlOptions>>,

    #[serde(rename = "outputTransmission")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_transmission: Option<Vec<models::TransmissionMode>>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<models::Link>>,
}

impl ProcessSummaryAllOf {
    pub fn new(id: String, version: String) -> ProcessSummaryAllOf {
        ProcessSummaryAllOf {
            id: id,
            version: version,
            job_control_options: None,
            output_transmission: None,
            links: None,
        }
    }
}

/// Converts the ProcessSummaryAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ProcessSummaryAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("id".to_string());
        params.push(self.id.to_string());

        params.push("version".to_string());
        params.push(self.version.to_string());

        // Skipping jobControlOptions in query parameter serialization

        // Skipping outputTransmission in query parameter serialization

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProcessSummaryAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProcessSummaryAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub id: Vec<String>,
            pub version: Vec<String>,
            pub job_control_options: Vec<Vec<models::JobControlOptions>>,
            pub output_transmission: Vec<Vec<models::TransmissionMode>>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ProcessSummaryAllOf".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "id" => intermediate_rep.id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "version" => intermediate_rep.version.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "jobControlOptions" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in ProcessSummaryAllOf"
                            .to_string(),
                    ),
                    "outputTransmission" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in ProcessSummaryAllOf"
                            .to_string(),
                    ),
                    "links" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in ProcessSummaryAllOf"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ProcessSummaryAllOf".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProcessSummaryAllOf {
            id: intermediate_rep
                .id
                .into_iter()
                .next()
                .ok_or("id missing in ProcessSummaryAllOf".to_string())?,
            version: intermediate_rep
                .version
                .into_iter()
                .next()
                .ok_or("version missing in ProcessSummaryAllOf".to_string())?,
            job_control_options: intermediate_rep.job_control_options.into_iter().next(),
            output_transmission: intermediate_rep.output_transmission.into_iter().next(),
            links: intermediate_rep.links.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProcessSummaryAllOf> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ProcessSummaryAllOf>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ProcessSummaryAllOf>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ProcessSummaryAllOf - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ProcessSummaryAllOf>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ProcessSummaryAllOf as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ProcessSummaryAllOf - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Reference {
    #[serde(rename = "$ref")]
    pub ref_: String,
}

impl Reference {
    pub fn new(ref_: String) -> Reference {
        Reference { ref_: ref_ }
    }
}

/// Converts the Reference value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Reference {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("$ref".to_string());
        params.push(self.ref_.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Reference value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Reference {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub ref_: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Reference".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "$ref" => intermediate_rep.ref_.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Reference".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Reference {
            ref_: intermediate_rep
                .ref_
                .into_iter()
                .next()
                .ok_or("$ref missing in Reference".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Reference> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Reference>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Reference>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Reference - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Reference> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Reference as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Reference - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Schema {
    #[serde(rename = "$ref")]
    pub ref_: String,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "multipleOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    #[serde(rename = "maximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    #[serde(rename = "exclusiveMaximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,

    #[serde(rename = "minimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(rename = "exclusiveMinimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,

    #[serde(rename = "maxLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(rename = "minLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(rename = "pattern")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(rename = "maxItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    #[serde(rename = "minItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    #[serde(rename = "uniqueItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    #[serde(rename = "maxProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,

    #[serde(rename = "minProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,

    #[serde(rename = "required")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<serde_json::Value>>,

    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    #[serde(rename = "not")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<swagger::OneOf2<schemaYaml, referenceYaml>>,

    #[serde(rename = "allOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "oneOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "anyOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "items")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<swagger::OneOf2<schemaYaml, referenceYaml>>,

    #[serde(rename = "properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties:
        Option<std::collections::HashMap<String, swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "additionalProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<swagger::OneOf3<schemaYaml, referenceYaml, bool>>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "format")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(rename = "default")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(rename = "nullable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    #[serde(rename = "readOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(rename = "writeOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    #[serde(rename = "example")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    #[serde(rename = "deprecated")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(rename = "contentMediaType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_media_type: Option<String>,

    #[serde(rename = "contentEncoding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    #[serde(rename = "contentSchema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_schema: Option<String>,
}

impl Schema {
    pub fn new(ref_: String) -> Schema {
        Schema {
            ref_: ref_,
            title: None,
            multiple_of: None,
            maximum: None,
            exclusive_maximum: Some(false),
            minimum: None,
            exclusive_minimum: Some(false),
            max_length: None,
            min_length: Some(0),
            pattern: None,
            max_items: None,
            min_items: Some(0),
            unique_items: Some(false),
            max_properties: None,
            min_properties: Some(0),
            required: None,
            enum_: None,
            type_: None,
            not: None,
            all_of: None,
            one_of: None,
            any_of: None,
            items: None,
            properties: None,
            additional_properties: None,
            description: None,
            format: None,
            default: None,
            nullable: Some(false),
            read_only: Some(false),
            write_only: Some(false),
            example: None,
            deprecated: Some(false),
            content_media_type: None,
            content_encoding: None,
            content_schema: None,
        }
    }
}

/// Converts the Schema value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Schema {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("$ref".to_string());
        params.push(self.ref_.to_string());

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref multiple_of) = self.multiple_of {
            params.push("multipleOf".to_string());
            params.push(multiple_of.to_string());
        }

        if let Some(ref maximum) = self.maximum {
            params.push("maximum".to_string());
            params.push(maximum.to_string());
        }

        if let Some(ref exclusive_maximum) = self.exclusive_maximum {
            params.push("exclusiveMaximum".to_string());
            params.push(exclusive_maximum.to_string());
        }

        if let Some(ref minimum) = self.minimum {
            params.push("minimum".to_string());
            params.push(minimum.to_string());
        }

        if let Some(ref exclusive_minimum) = self.exclusive_minimum {
            params.push("exclusiveMinimum".to_string());
            params.push(exclusive_minimum.to_string());
        }

        if let Some(ref max_length) = self.max_length {
            params.push("maxLength".to_string());
            params.push(max_length.to_string());
        }

        if let Some(ref min_length) = self.min_length {
            params.push("minLength".to_string());
            params.push(min_length.to_string());
        }

        if let Some(ref pattern) = self.pattern {
            params.push("pattern".to_string());
            params.push(pattern.to_string());
        }

        if let Some(ref max_items) = self.max_items {
            params.push("maxItems".to_string());
            params.push(max_items.to_string());
        }

        if let Some(ref min_items) = self.min_items {
            params.push("minItems".to_string());
            params.push(min_items.to_string());
        }

        if let Some(ref unique_items) = self.unique_items {
            params.push("uniqueItems".to_string());
            params.push(unique_items.to_string());
        }

        if let Some(ref max_properties) = self.max_properties {
            params.push("maxProperties".to_string());
            params.push(max_properties.to_string());
        }

        if let Some(ref min_properties) = self.min_properties {
            params.push("minProperties".to_string());
            params.push(min_properties.to_string());
        }

        if let Some(ref required) = self.required {
            params.push("required".to_string());
            params.push(
                required
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping enum in query parameter serialization

        if let Some(ref type_) = self.type_ {
            params.push("type".to_string());
            params.push(type_.to_string());
        }

        // Skipping not in query parameter serialization

        // Skipping allOf in query parameter serialization

        // Skipping oneOf in query parameter serialization

        // Skipping anyOf in query parameter serialization

        // Skipping items in query parameter serialization

        // Skipping properties in query parameter serialization
        // Skipping properties in query parameter serialization

        // Skipping additionalProperties in query parameter serialization

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref format) = self.format {
            params.push("format".to_string());
            params.push(format.to_string());
        }

        // Skipping default in query parameter serialization

        if let Some(ref nullable) = self.nullable {
            params.push("nullable".to_string());
            params.push(nullable.to_string());
        }

        if let Some(ref read_only) = self.read_only {
            params.push("readOnly".to_string());
            params.push(read_only.to_string());
        }

        if let Some(ref write_only) = self.write_only {
            params.push("writeOnly".to_string());
            params.push(write_only.to_string());
        }

        // Skipping example in query parameter serialization

        if let Some(ref deprecated) = self.deprecated {
            params.push("deprecated".to_string());
            params.push(deprecated.to_string());
        }

        if let Some(ref content_media_type) = self.content_media_type {
            params.push("contentMediaType".to_string());
            params.push(content_media_type.to_string());
        }

        if let Some(ref content_encoding) = self.content_encoding {
            params.push("contentEncoding".to_string());
            params.push(content_encoding.to_string());
        }

        if let Some(ref content_schema) = self.content_schema {
            params.push("contentSchema".to_string());
            params.push(content_schema.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Schema value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Schema {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub ref_: Vec<String>,
            pub title: Vec<String>,
            pub multiple_of: Vec<f64>,
            pub maximum: Vec<f64>,
            pub exclusive_maximum: Vec<bool>,
            pub minimum: Vec<f64>,
            pub exclusive_minimum: Vec<bool>,
            pub max_length: Vec<usize>,
            pub min_length: Vec<usize>,
            pub pattern: Vec<String>,
            pub max_items: Vec<usize>,
            pub min_items: Vec<usize>,
            pub unique_items: Vec<bool>,
            pub max_properties: Vec<usize>,
            pub min_properties: Vec<usize>,
            pub required: Vec<Vec<String>>,
            pub enum_: Vec<Vec<serde_json::Value>>,
            pub type_: Vec<String>,
            pub not: Vec<swagger::OneOf2<schemaYaml, referenceYaml>>,
            pub all_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub one_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub any_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub items: Vec<swagger::OneOf2<schemaYaml, referenceYaml>>,
            pub properties:
                Vec<std::collections::HashMap<String, swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub additional_properties: Vec<swagger::OneOf3<schemaYaml, referenceYaml, bool>>,
            pub description: Vec<String>,
            pub format: Vec<String>,
            pub default: Vec<serde_json::Value>,
            pub nullable: Vec<bool>,
            pub read_only: Vec<bool>,
            pub write_only: Vec<bool>,
            pub example: Vec<serde_json::Value>,
            pub deprecated: Vec<bool>,
            pub content_media_type: Vec<String>,
            pub content_encoding: Vec<String>,
            pub content_schema: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Schema".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "$ref" => intermediate_rep.ref_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "multipleOf" => intermediate_rep.multiple_of.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maximum" => intermediate_rep.maximum.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "exclusiveMaximum" => intermediate_rep.exclusive_maximum.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minimum" => intermediate_rep.minimum.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "exclusiveMinimum" => intermediate_rep.exclusive_minimum.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxLength" => intermediate_rep.max_length.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minLength" => intermediate_rep.min_length.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "pattern" => intermediate_rep.pattern.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxItems" => intermediate_rep.max_items.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minItems" => intermediate_rep.min_items.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "uniqueItems" => intermediate_rep.unique_items.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxProperties" => intermediate_rep.max_properties.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minProperties" => intermediate_rep.min_properties.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "required" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "enum" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "not" => intermediate_rep.not.push(<swagger::OneOf2<schemaYaml,referenceYaml> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "allOf" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "oneOf" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "anyOf" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "items" => intermediate_rep.items.push(<swagger::OneOf2<schemaYaml,referenceYaml> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "properties" => return std::result::Result::Err("Parsing a container in this style is not supported in Schema".to_string()),
                    "additionalProperties" => intermediate_rep.additional_properties.push(<swagger::OneOf3<schemaYaml,referenceYaml,bool> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "description" => intermediate_rep.description.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "format" => intermediate_rep.format.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "default" => intermediate_rep.default.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nullable" => intermediate_rep.nullable.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "readOnly" => intermediate_rep.read_only.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "writeOnly" => intermediate_rep.write_only.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "example" => intermediate_rep.example.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "deprecated" => intermediate_rep.deprecated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentMediaType" => intermediate_rep.content_media_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentEncoding" => intermediate_rep.content_encoding.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentSchema" => intermediate_rep.content_schema.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Schema".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Schema {
            ref_: intermediate_rep
                .ref_
                .into_iter()
                .next()
                .ok_or("$ref missing in Schema".to_string())?,
            title: intermediate_rep.title.into_iter().next(),
            multiple_of: intermediate_rep.multiple_of.into_iter().next(),
            maximum: intermediate_rep.maximum.into_iter().next(),
            exclusive_maximum: intermediate_rep.exclusive_maximum.into_iter().next(),
            minimum: intermediate_rep.minimum.into_iter().next(),
            exclusive_minimum: intermediate_rep.exclusive_minimum.into_iter().next(),
            max_length: intermediate_rep.max_length.into_iter().next(),
            min_length: intermediate_rep.min_length.into_iter().next(),
            pattern: intermediate_rep.pattern.into_iter().next(),
            max_items: intermediate_rep.max_items.into_iter().next(),
            min_items: intermediate_rep.min_items.into_iter().next(),
            unique_items: intermediate_rep.unique_items.into_iter().next(),
            max_properties: intermediate_rep.max_properties.into_iter().next(),
            min_properties: intermediate_rep.min_properties.into_iter().next(),
            required: intermediate_rep.required.into_iter().next(),
            enum_: intermediate_rep.enum_.into_iter().next(),
            type_: intermediate_rep.type_.into_iter().next(),
            not: intermediate_rep.not.into_iter().next(),
            all_of: intermediate_rep.all_of.into_iter().next(),
            one_of: intermediate_rep.one_of.into_iter().next(),
            any_of: intermediate_rep.any_of.into_iter().next(),
            items: intermediate_rep.items.into_iter().next(),
            properties: intermediate_rep.properties.into_iter().next(),
            additional_properties: intermediate_rep.additional_properties.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            format: intermediate_rep.format.into_iter().next(),
            default: intermediate_rep.default.into_iter().next(),
            nullable: intermediate_rep.nullable.into_iter().next(),
            read_only: intermediate_rep.read_only.into_iter().next(),
            write_only: intermediate_rep.write_only.into_iter().next(),
            example: intermediate_rep.example.into_iter().next(),
            deprecated: intermediate_rep.deprecated.into_iter().next(),
            content_media_type: intermediate_rep.content_media_type.into_iter().next(),
            content_encoding: intermediate_rep.content_encoding.into_iter().next(),
            content_schema: intermediate_rep.content_schema.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Schema> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Schema>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Schema>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Schema - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Schema> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Schema as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Schema - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SchemaOneOf {
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "multipleOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    #[serde(rename = "maximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    #[serde(rename = "exclusiveMaximum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,

    #[serde(rename = "minimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(rename = "exclusiveMinimum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,

    #[serde(rename = "maxLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(rename = "minLength")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(rename = "pattern")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(rename = "maxItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    #[serde(rename = "minItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    #[serde(rename = "uniqueItems")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    #[serde(rename = "maxProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,

    #[serde(rename = "minProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,

    #[serde(rename = "required")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<serde_json::Value>>,

    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    #[serde(rename = "not")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<swagger::OneOf2<schemaYaml, referenceYaml>>,

    #[serde(rename = "allOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "oneOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "anyOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "items")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<swagger::OneOf2<schemaYaml, referenceYaml>>,

    #[serde(rename = "properties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties:
        Option<std::collections::HashMap<String, swagger::OneOf2<schemaYaml, referenceYaml>>>,

    #[serde(rename = "additionalProperties")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<swagger::OneOf3<schemaYaml, referenceYaml, bool>>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "format")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(rename = "default")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(rename = "nullable")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    #[serde(rename = "readOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,

    #[serde(rename = "writeOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_only: Option<bool>,

    #[serde(rename = "example")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    #[serde(rename = "deprecated")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    #[serde(rename = "contentMediaType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_media_type: Option<String>,

    #[serde(rename = "contentEncoding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,

    #[serde(rename = "contentSchema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_schema: Option<String>,
}

impl SchemaOneOf {
    pub fn new() -> SchemaOneOf {
        SchemaOneOf {
            title: None,
            multiple_of: None,
            maximum: None,
            exclusive_maximum: Some(false),
            minimum: None,
            exclusive_minimum: Some(false),
            max_length: None,
            min_length: Some(0),
            pattern: None,
            max_items: None,
            min_items: Some(0),
            unique_items: Some(false),
            max_properties: None,
            min_properties: Some(0),
            required: None,
            enum_: None,
            type_: None,
            not: None,
            all_of: None,
            one_of: None,
            any_of: None,
            items: None,
            properties: None,
            additional_properties: None,
            description: None,
            format: None,
            default: None,
            nullable: Some(false),
            read_only: Some(false),
            write_only: Some(false),
            example: None,
            deprecated: Some(false),
            content_media_type: None,
            content_encoding: None,
            content_schema: None,
        }
    }
}

/// Converts the SchemaOneOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SchemaOneOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref multiple_of) = self.multiple_of {
            params.push("multipleOf".to_string());
            params.push(multiple_of.to_string());
        }

        if let Some(ref maximum) = self.maximum {
            params.push("maximum".to_string());
            params.push(maximum.to_string());
        }

        if let Some(ref exclusive_maximum) = self.exclusive_maximum {
            params.push("exclusiveMaximum".to_string());
            params.push(exclusive_maximum.to_string());
        }

        if let Some(ref minimum) = self.minimum {
            params.push("minimum".to_string());
            params.push(minimum.to_string());
        }

        if let Some(ref exclusive_minimum) = self.exclusive_minimum {
            params.push("exclusiveMinimum".to_string());
            params.push(exclusive_minimum.to_string());
        }

        if let Some(ref max_length) = self.max_length {
            params.push("maxLength".to_string());
            params.push(max_length.to_string());
        }

        if let Some(ref min_length) = self.min_length {
            params.push("minLength".to_string());
            params.push(min_length.to_string());
        }

        if let Some(ref pattern) = self.pattern {
            params.push("pattern".to_string());
            params.push(pattern.to_string());
        }

        if let Some(ref max_items) = self.max_items {
            params.push("maxItems".to_string());
            params.push(max_items.to_string());
        }

        if let Some(ref min_items) = self.min_items {
            params.push("minItems".to_string());
            params.push(min_items.to_string());
        }

        if let Some(ref unique_items) = self.unique_items {
            params.push("uniqueItems".to_string());
            params.push(unique_items.to_string());
        }

        if let Some(ref max_properties) = self.max_properties {
            params.push("maxProperties".to_string());
            params.push(max_properties.to_string());
        }

        if let Some(ref min_properties) = self.min_properties {
            params.push("minProperties".to_string());
            params.push(min_properties.to_string());
        }

        if let Some(ref required) = self.required {
            params.push("required".to_string());
            params.push(
                required
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping enum in query parameter serialization

        if let Some(ref type_) = self.type_ {
            params.push("type".to_string());
            params.push(type_.to_string());
        }

        // Skipping not in query parameter serialization

        // Skipping allOf in query parameter serialization

        // Skipping oneOf in query parameter serialization

        // Skipping anyOf in query parameter serialization

        // Skipping items in query parameter serialization

        // Skipping properties in query parameter serialization
        // Skipping properties in query parameter serialization

        // Skipping additionalProperties in query parameter serialization

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        if let Some(ref format) = self.format {
            params.push("format".to_string());
            params.push(format.to_string());
        }

        // Skipping default in query parameter serialization

        if let Some(ref nullable) = self.nullable {
            params.push("nullable".to_string());
            params.push(nullable.to_string());
        }

        if let Some(ref read_only) = self.read_only {
            params.push("readOnly".to_string());
            params.push(read_only.to_string());
        }

        if let Some(ref write_only) = self.write_only {
            params.push("writeOnly".to_string());
            params.push(write_only.to_string());
        }

        // Skipping example in query parameter serialization

        if let Some(ref deprecated) = self.deprecated {
            params.push("deprecated".to_string());
            params.push(deprecated.to_string());
        }

        if let Some(ref content_media_type) = self.content_media_type {
            params.push("contentMediaType".to_string());
            params.push(content_media_type.to_string());
        }

        if let Some(ref content_encoding) = self.content_encoding {
            params.push("contentEncoding".to_string());
            params.push(content_encoding.to_string());
        }

        if let Some(ref content_schema) = self.content_schema {
            params.push("contentSchema".to_string());
            params.push(content_schema.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SchemaOneOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SchemaOneOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub title: Vec<String>,
            pub multiple_of: Vec<f64>,
            pub maximum: Vec<f64>,
            pub exclusive_maximum: Vec<bool>,
            pub minimum: Vec<f64>,
            pub exclusive_minimum: Vec<bool>,
            pub max_length: Vec<usize>,
            pub min_length: Vec<usize>,
            pub pattern: Vec<String>,
            pub max_items: Vec<usize>,
            pub min_items: Vec<usize>,
            pub unique_items: Vec<bool>,
            pub max_properties: Vec<usize>,
            pub min_properties: Vec<usize>,
            pub required: Vec<Vec<String>>,
            pub enum_: Vec<Vec<serde_json::Value>>,
            pub type_: Vec<String>,
            pub not: Vec<swagger::OneOf2<schemaYaml, referenceYaml>>,
            pub all_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub one_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub any_of: Vec<Vec<swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub items: Vec<swagger::OneOf2<schemaYaml, referenceYaml>>,
            pub properties:
                Vec<std::collections::HashMap<String, swagger::OneOf2<schemaYaml, referenceYaml>>>,
            pub additional_properties: Vec<swagger::OneOf3<schemaYaml, referenceYaml, bool>>,
            pub description: Vec<String>,
            pub format: Vec<String>,
            pub default: Vec<serde_json::Value>,
            pub nullable: Vec<bool>,
            pub read_only: Vec<bool>,
            pub write_only: Vec<bool>,
            pub example: Vec<serde_json::Value>,
            pub deprecated: Vec<bool>,
            pub content_media_type: Vec<String>,
            pub content_encoding: Vec<String>,
            pub content_schema: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing SchemaOneOf".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "multipleOf" => intermediate_rep.multiple_of.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maximum" => intermediate_rep.maximum.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "exclusiveMaximum" => intermediate_rep.exclusive_maximum.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minimum" => intermediate_rep.minimum.push(<f64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "exclusiveMinimum" => intermediate_rep.exclusive_minimum.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxLength" => intermediate_rep.max_length.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minLength" => intermediate_rep.min_length.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "pattern" => intermediate_rep.pattern.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxItems" => intermediate_rep.max_items.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minItems" => intermediate_rep.min_items.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "uniqueItems" => intermediate_rep.unique_items.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "maxProperties" => intermediate_rep.max_properties.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "minProperties" => intermediate_rep.min_properties.push(<usize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "required" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "enum" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "not" => intermediate_rep.not.push(<swagger::OneOf2<schemaYaml,referenceYaml> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "allOf" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "oneOf" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "anyOf" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "items" => intermediate_rep.items.push(<swagger::OneOf2<schemaYaml,referenceYaml> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "properties" => return std::result::Result::Err("Parsing a container in this style is not supported in SchemaOneOf".to_string()),
                    "additionalProperties" => intermediate_rep.additional_properties.push(<swagger::OneOf3<schemaYaml,referenceYaml,bool> as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "description" => intermediate_rep.description.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "format" => intermediate_rep.format.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "default" => intermediate_rep.default.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nullable" => intermediate_rep.nullable.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "readOnly" => intermediate_rep.read_only.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "writeOnly" => intermediate_rep.write_only.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "example" => intermediate_rep.example.push(<serde_json::Value as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "deprecated" => intermediate_rep.deprecated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentMediaType" => intermediate_rep.content_media_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentEncoding" => intermediate_rep.content_encoding.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "contentSchema" => intermediate_rep.content_schema.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing SchemaOneOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SchemaOneOf {
            title: intermediate_rep.title.into_iter().next(),
            multiple_of: intermediate_rep.multiple_of.into_iter().next(),
            maximum: intermediate_rep.maximum.into_iter().next(),
            exclusive_maximum: intermediate_rep.exclusive_maximum.into_iter().next(),
            minimum: intermediate_rep.minimum.into_iter().next(),
            exclusive_minimum: intermediate_rep.exclusive_minimum.into_iter().next(),
            max_length: intermediate_rep.max_length.into_iter().next(),
            min_length: intermediate_rep.min_length.into_iter().next(),
            pattern: intermediate_rep.pattern.into_iter().next(),
            max_items: intermediate_rep.max_items.into_iter().next(),
            min_items: intermediate_rep.min_items.into_iter().next(),
            unique_items: intermediate_rep.unique_items.into_iter().next(),
            max_properties: intermediate_rep.max_properties.into_iter().next(),
            min_properties: intermediate_rep.min_properties.into_iter().next(),
            required: intermediate_rep.required.into_iter().next(),
            enum_: intermediate_rep.enum_.into_iter().next(),
            type_: intermediate_rep.type_.into_iter().next(),
            not: intermediate_rep.not.into_iter().next(),
            all_of: intermediate_rep.all_of.into_iter().next(),
            one_of: intermediate_rep.one_of.into_iter().next(),
            any_of: intermediate_rep.any_of.into_iter().next(),
            items: intermediate_rep.items.into_iter().next(),
            properties: intermediate_rep.properties.into_iter().next(),
            additional_properties: intermediate_rep.additional_properties.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            format: intermediate_rep.format.into_iter().next(),
            default: intermediate_rep.default.into_iter().next(),
            nullable: intermediate_rep.nullable.into_iter().next(),
            read_only: intermediate_rep.read_only.into_iter().next(),
            write_only: intermediate_rep.write_only.into_iter().next(),
            example: intermediate_rep.example.into_iter().next(),
            deprecated: intermediate_rep.deprecated.into_iter().next(),
            content_media_type: intermediate_rep.content_media_type.into_iter().next(),
            content_encoding: intermediate_rep.content_encoding.into_iter().next(),
            content_schema: intermediate_rep.content_schema.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SchemaOneOf> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<SchemaOneOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<SchemaOneOf>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for SchemaOneOf - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SchemaOneOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <SchemaOneOf as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into SchemaOneOf - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum StatusCode {
    #[serde(rename = "accepted")]
    ACCEPTED,
    #[serde(rename = "running")]
    RUNNING,
    #[serde(rename = "successful")]
    SUCCESSFUL,
    #[serde(rename = "failed")]
    FAILED,
    #[serde(rename = "dismissed")]
    DISMISSED,
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StatusCode::ACCEPTED => write!(f, "{}", "accepted"),
            StatusCode::RUNNING => write!(f, "{}", "running"),
            StatusCode::SUCCESSFUL => write!(f, "{}", "successful"),
            StatusCode::FAILED => write!(f, "{}", "failed"),
            StatusCode::DISMISSED => write!(f, "{}", "dismissed"),
        }
    }
}

impl std::str::FromStr for StatusCode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "accepted" => std::result::Result::Ok(StatusCode::ACCEPTED),
            "running" => std::result::Result::Ok(StatusCode::RUNNING),
            "successful" => std::result::Result::Ok(StatusCode::SUCCESSFUL),
            "failed" => std::result::Result::Ok(StatusCode::FAILED),
            "dismissed" => std::result::Result::Ok(StatusCode::DISMISSED),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StatusInfo {
    #[serde(rename = "processID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<String>,

    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "jobID")]
    pub job_id: String,

    #[serde(rename = "status")]
    pub status: models::StatusCode,

    #[serde(rename = "message")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "started")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "finished")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "updated")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "progress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u8>,

    #[serde(rename = "links")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<models::Link>>,
}

impl StatusInfo {
    pub fn new(type_: String, job_id: String, status: models::StatusCode) -> StatusInfo {
        StatusInfo {
            process_id: None,
            type_: type_,
            job_id: job_id,
            status: status,
            message: None,
            created: None,
            started: None,
            finished: None,
            updated: None,
            progress: None,
            links: None,
        }
    }
}

/// Converts the StatusInfo value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for StatusInfo {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref process_id) = self.process_id {
            params.push("processID".to_string());
            params.push(process_id.to_string());
        }

        params.push("type".to_string());
        params.push(self.type_.to_string());

        params.push("jobID".to_string());
        params.push(self.job_id.to_string());

        // Skipping status in query parameter serialization

        if let Some(ref message) = self.message {
            params.push("message".to_string());
            params.push(message.to_string());
        }

        // Skipping created in query parameter serialization

        // Skipping started in query parameter serialization

        // Skipping finished in query parameter serialization

        // Skipping updated in query parameter serialization

        if let Some(ref progress) = self.progress {
            params.push("progress".to_string());
            params.push(progress.to_string());
        }

        // Skipping links in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a StatusInfo value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for StatusInfo {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub process_id: Vec<String>,
            pub type_: Vec<String>,
            pub job_id: Vec<String>,
            pub status: Vec<models::StatusCode>,
            pub message: Vec<String>,
            pub created: Vec<chrono::DateTime<chrono::Utc>>,
            pub started: Vec<chrono::DateTime<chrono::Utc>>,
            pub finished: Vec<chrono::DateTime<chrono::Utc>>,
            pub updated: Vec<chrono::DateTime<chrono::Utc>>,
            pub progress: Vec<u8>,
            pub links: Vec<Vec<models::Link>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing StatusInfo".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "processID" => intermediate_rep.process_id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "type" => intermediate_rep.type_.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "jobID" => intermediate_rep.job_id.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "status" => intermediate_rep.status.push(
                        <models::StatusCode as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "message" => intermediate_rep.message.push(
                        <String as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "created" => intermediate_rep.created.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "started" => intermediate_rep.started.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "finished" => intermediate_rep.finished.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "updated" => intermediate_rep.updated.push(
                        <chrono::DateTime<chrono::Utc> as std::str::FromStr>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "progress" => intermediate_rep.progress.push(
                        <u8 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "links" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in StatusInfo"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing StatusInfo".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(StatusInfo {
            process_id: intermediate_rep.process_id.into_iter().next(),
            type_: intermediate_rep
                .type_
                .into_iter()
                .next()
                .ok_or("type missing in StatusInfo".to_string())?,
            job_id: intermediate_rep
                .job_id
                .into_iter()
                .next()
                .ok_or("jobID missing in StatusInfo".to_string())?,
            status: intermediate_rep
                .status
                .into_iter()
                .next()
                .ok_or("status missing in StatusInfo".to_string())?,
            message: intermediate_rep.message.into_iter().next(),
            created: intermediate_rep.created.into_iter().next(),
            started: intermediate_rep.started.into_iter().next(),
            finished: intermediate_rep.finished.into_iter().next(),
            updated: intermediate_rep.updated.into_iter().next(),
            progress: intermediate_rep.progress.into_iter().next(),
            links: intermediate_rep.links.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<StatusInfo> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<StatusInfo>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<StatusInfo>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for StatusInfo - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<StatusInfo> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <StatusInfo as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into StatusInfo - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum TransmissionMode {
    #[serde(rename = "value")]
    VALUE,
    #[serde(rename = "reference")]
    REFERENCE,
}

impl std::fmt::Display for TransmissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransmissionMode::VALUE => write!(f, "{}", "value"),
            TransmissionMode::REFERENCE => write!(f, "{}", "reference"),
        }
    }
}

impl std::str::FromStr for TransmissionMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "value" => std::result::Result::Ok(TransmissionMode::VALUE),
            "reference" => std::result::Result::Ok(TransmissionMode::REFERENCE),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

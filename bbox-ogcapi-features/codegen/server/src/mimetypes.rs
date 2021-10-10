/// mime types for requests and responses

pub mod responses {
    use hyper::mime::*;

    // The macro is called per-operation to beat the recursion limit

    lazy_static! {
        /// Create Mime objects for the response content types for DescribeCollection
        pub static ref DESCRIBE_COLLECTION_INFORMATION_ABOUT_THE_FEATURE_COLLECTION_WITH_ID: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for DescribeCollection
        pub static ref DESCRIBE_COLLECTION_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetCollections
        pub static ref GET_COLLECTIONS_THE_FEATURE_COLLECTIONS_SHARED_BY_THIS_API: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetCollections
        pub static ref GET_COLLECTIONS_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetConformanceDeclaration
        pub static ref GET_CONFORMANCE_DECLARATION_THE_UR_IS_OF_ALL_CONFORMANCE_CLASSES_SUPPORTED_BY_THE_SERVER: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetConformanceDeclaration
        pub static ref GET_CONFORMANCE_DECLARATION_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetLandingPage
        pub static ref GET_LANDING_PAGE_THE_LANDING_PAGE_PROVIDES_LINKS_TO_THE_API_DEFINITION: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetLandingPage
        pub static ref GET_LANDING_PAGE_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetFeature
        pub static ref GET_FEATURE_FETCH_THE_FEATURE_WITH_ID: Mime = "application/geo+json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetFeature
        pub static ref GET_FEATURE_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetFeatures
        pub static ref GET_FEATURES_THE_RESPONSE_IS_A_DOCUMENT_CONSISTING_OF_FEATURES_IN_THE_COLLECTION: Mime = "application/geo+json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetFeatures
        pub static ref GET_FEATURES_A_QUERY_PARAMETER_HAS_AN_INVALID_VALUE: Mime = "application/json".parse().unwrap();
    }

    lazy_static! {
        /// Create Mime objects for the response content types for GetFeatures
        pub static ref GET_FEATURES_A_SERVER_ERROR_OCCURRED: Mime = "application/json".parse().unwrap();
    }

}

pub mod requests {
    use hyper::mime::*;

}

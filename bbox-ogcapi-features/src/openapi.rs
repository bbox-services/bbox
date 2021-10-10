pub const OPEN_API_TEMPLATE: &str = r#"{
  "openapi": "3.0.2",
  "info": {
    "title": "OGC API - Features",
    "version": "1.0.0",
    "description": "This is an OpenAPI definition that conforms to the conformance\nclasses \"Core\", \"GeoJSON\" and \"OpenAPI 3.0\" of the\nstandard \"OGC API - Features - Part 1: Core\".",
    "contact": {
      "name": "Acme Corporation",
      "email": "info@example.org",
      "url": "http://example.org/"
    },
    "license": {
      "name": "CC-BY 4.0 license",
      "url": "https://creativecommons.org/licenses/by/4.0/"
    }
  },
  "servers": [
    {
      "url": "https://data.example.org/",
      "description": "t-rex server"
    }
  ],
  "tags": [
    {
      "name": "Capabilities",
      "description": "essential characteristics of this API"
    },
    {
      "name": "Data",
      "description": "access to data (features)"
    }
  ],
  "paths": {
    "/": {
      "get": {
        "tags": [
          "Capabilities"
        ],
        "summary": "landing page",
        "description": "The landing page provides links to the API definition, the conformance\nstatements and to the feature collections in this dataset.",
        "operationId": "getLandingPage",
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/LandingPage"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    },
    "/conformance": {
      "get": {
        "tags": [
          "Capabilities"
        ],
        "summary": "information about specifications that this API conforms to",
        "description": "A list of all conformance classes specified in a standard that the\nserver conforms to.",
        "operationId": "getConformanceDeclaration",
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ConformanceDeclaration"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    },
    "/collections": {
      "get": {
        "tags": [
          "Capabilities"
        ],
        "summary": "the feature collections in the dataset",
        "operationId": "getCollections",
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Collections"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    },
    "/collections/{collectionId}": {
      "get": {
        "tags": [
          "Capabilities"
        ],
        "summary": "describe the feature collection with id `collectionId`",
        "operationId": "describeCollection",
        "parameters": [
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
          }
        ],
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Collection"
          },
          "404": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    },
    "/collections/{collectionId}/items": {
      "get": {
        "tags": [
          "Data"
        ],
        "summary": "fetch features",
        "description": "Fetch features of the feature collection with id `collectionId`.\n\nEvery feature in a dataset belongs to a collection. A dataset may\nconsist of multiple feature collections. A feature collection is often a\ncollection of features of a similar type, based on a common schema.\n\nUse content negotiation to request HTML or GeoJSON.",
        "operationId": "getFeatures",
        "parameters": [
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
          },
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/limit"
          },
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/bbox"
          },
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/datetime"
          }
        ],
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Features"
          },
          "400": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/InvalidParameter"
          },
          "404": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    },
    "/collections/{collectionId}/items/{featureId}": {
      "get": {
        "tags": [
          "Data"
        ],
        "summary": "fetch a single feature",
        "description": "Fetch the feature with id `featureId` in the feature collection\nwith id `collectionId`.\n\nUse content negotiation to request HTML or GeoJSON.",
        "operationId": "getFeature",
        "parameters": [
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/collectionId"
          },
          {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/parameters/featureId"
          }
        ],
        "responses": {
          "200": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Feature"
          },
          "404": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/NotFound"
          },
          "500": {
            "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
          }
        }
      }
    }
  }
}
"#;

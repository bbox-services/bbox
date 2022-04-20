use actix_web::{web, HttpRequest, HttpResponse};
use bbox_common::api::OgcApiInventory;
use bbox_common::ogcapi::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

fn relurl(req: &HttpRequest, path: &str) -> String {
    let conninfo = req.connection_info();
    let pathbase = path.split('/').nth(1).unwrap_or("");
    let reqbase = req
        .path()
        .split('/')
        .nth(1)
        .map(|p| {
            if p == "" || p == pathbase {
                "".to_string()
            } else {
                format!("/{}", p)
            }
        })
        .unwrap_or("".to_string());
    format!(
        "{}://{}{}{}",
        conninfo.scheme(),
        conninfo.host(),
        reqbase,
        path
    )
}

/// landing page
///
/// The landing page provides links to the API definition, the conformance
/// statements and to the feature collections in this dataset.
#[utoipa::path(
    get,
    path = "/ogcapi/",
    operation_id = "getLandingPage",
    tag = "Capabilities",
    responses(
        (status = 200, body = CoreLandingPage), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/LandingPage"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    )
)]
async fn index(ogcapi: web::Data<OgcApiInventory>, _req: HttpRequest) -> HttpResponse {
    let links = ogcapi.landing_page_links.to_vec(); //TODO: convert urls with relurl (?)
    let landing_page = CoreLandingPage {
        title: Some("BBOX OGC API".to_string()),
        description: Some("BBOX OGC API landing page".to_string()),
        links,
    };
    HttpResponse::Ok().json(landing_page)
}

/// information about specifications that this API conforms to
///
/// A list of all conformance classes specified in a standard that the
/// server conforms to.
#[utoipa::path(
    get,
    path = "/conformance",
    operation_id = "getConformanceDeclaration",
    tag = "Capabilities",
    responses(
        (status = 200, body = CoreConformsTo), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ConformanceDeclaration"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    )
)]
async fn conformance(ogcapi: web::Data<OgcApiInventory>) -> HttpResponse {
    let conforms_to = CoreConformsTo {
        conforms_to: ogcapi.conformance_classes.to_vec(),
    };
    HttpResponse::Ok().json(conforms_to)
}

/// the feature collections in the dataset
///
/// A list of all conformance classes specified in a standard that the
/// server conforms to.
#[utoipa::path(
    get,
    path = "/collections",
    operation_id = "getCollections",
    tag = "Capabilities",
    responses(
        (status = 200, body = CoreCollections), // "$ref": https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/Collections"
        (status = 500), // "$ref": "https://raw.githubusercontent.com/opengeospatial/ogcapi-features/master/core/openapi/ogcapi-features-1.yaml#/components/responses/ServerError"
    )
)]
async fn collections(req: HttpRequest) -> HttpResponse {
    let collection = CoreCollection {
        id: "buildings".to_string(),
        title: Some("Buildings".to_string()),
        description: Some("Buildings in the city of Bonn.".to_string()),
        extent: Some(CoreExtent {
            spatial: Some(CoreExtentSpatial {
                bbox: vec![vec![7.01, 50.63, 7.22, 50.78]],
                crs: None,
            }),
            temporal: Some(CoreExtentTemporal {
                interval: vec![vec![Some("2010-02-15T12:34:56Z".to_string()), None]],
                trs: None,
            }),
        }),
        item_type: None,
        crs: vec![],
        links: vec![
            ApiLink {
                href: relurl(&req, "/collections/buildings/items"),
                rel: Some("items".to_string()),
                type_: Some("application/geo+json".to_string()),
                title: Some("Buildings".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                href: "https://creativecommons.org/publicdomain/zero/1.0/".to_string(),
                rel: Some("license".to_string()),
                type_: Some("text/html".to_string()),
                title: Some("CC0-1.0".to_string()),
                hreflang: None,
                length: None,
            },
            ApiLink {
                href: "https://creativecommons.org/publicdomain/zero/1.0/rdf".to_string(),
                rel: Some("license".to_string()),
                type_: Some("application/rdf+xml".to_string()),
                title: Some("CC0-1.0".to_string()),
                hreflang: None,
                length: None,
            },
        ],
    };
    let collections = CoreCollections {
        links: vec![ApiLink {
            href: relurl(&req, "/collections"),
            rel: Some("self".to_string()),
            type_: Some("application/json".to_string()),
            title: Some("this document".to_string()),
            hreflang: None,
            length: None,
        }],
        collections: vec![collection],
    };
    HttpResponse::Ok().json(collections)
}

#[derive(OpenApi)]
#[openapi(
    handlers(index, conformance, collections),
    components(CoreLandingPage,ApiLink,CoreConformsTo,CoreCollections,CoreCollection,CoreExtent,CoreExtentSpatial,CoreExtentTemporal,CoreFeatures,CoreFeature),
    tags(
        (name = "Capabilities", description = "essential characteristics of this API")
    ),
  // "info": {
  //   "title": "OGC API - Features",
  //   "version": "1.0.0",
  //   "description": "This is an OpenAPI definition that conforms to the conformance\nclasses \"Core\", \"GeoJSON\" and \"OpenAPI 3.0\" of the\nstandard \"OGC API - Features - Part 1: Core\".",
  //   "contact": {
  //     "name": "Acme Corporation",
  //     "email": "info@example.org",
  //     "url": "http://example.org/"
  //   },
  //   "license": {
  //     "name": "CC-BY 4.0 license",
  //     "url": "https://creativecommons.org/licenses/by/4.0/"
  //   }
  // },
  // "servers": [
  //   {
  //     "url": "https://data.example.org/",
  //     "description": "BBOX feature server"
  //   }
  // ],
)]
pub struct ApiDoc;

pub fn register(cfg: &mut web::ServiceConfig, apidoc: utoipa::openapi::OpenApi) {
    cfg.service(web::resource("/ogcapi/").route(web::get().to(index)))
        .service(SwaggerUi::new("/openapi/{_:.*}").url("/api", apidoc))
        .service(web::resource("/conformance").route(web::get().to(conformance)))
        .service(web::resource("/collections").route(web::get().to(collections)));
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use actix_web::{body, http, test, Error};

//     #[actix_web::test]
//     async fn test_index() -> Result<(), Error> {
//         let req = test::TestRequest::default().to_http_request();
//         let resp = index(req).await;

//         assert_eq!(resp.status(), http::StatusCode::OK);

//         let response_body = body::to_bytes(resp.into_body()).await?;

//         assert_eq!(response_body, "{\"title\":\"Buildings in Bonn\",\"description\":\"Access to data about buildings in the city of Bonn via a Web API that conforms to the OGC API Features specification\",\"links\":[{\"href\":\"http://localhost:8080/\",\"rel\":\"self\",\"type\":\"application/json\",\"title\":\"this document\"},{\"href\":\"http://localhost:8080/api\",\"rel\":\"service-desc\",\"type\":\"application/vnd.oai.openapi+json;version=3.0\",\"title\":\"the API definition\"},{\"href\":\"http://localhost:8080/conformance\",\"rel\":\"conformance\",\"type\":\"application/json\",\"title\":\"OGC API conformance classes implemented by this server\"},{\"href\":\"http://localhost:8080/collections\",\"rel\":\"data\",\"type\":\"application/json\",\"title\":\"Information about the feature collections\"}]}");

//         Ok(())
//     }
// }

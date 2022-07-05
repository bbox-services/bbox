use actix_web::{web, HttpResponse};
use bbox_common::api::{OgcApiInventory, OpenApiDoc, OpenApiDocCollection};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// The definition of the route to compute.
#[derive(Debug, Deserialize, Serialize)]
pub struct RouteDefinition {
    pub name: Option<String>,
    pub preference: Option<String>,
    pub waypoints: Waypoints,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Waypoints {
    pub coordinates: Vec<Vec<f64>>,
    #[serde(rename = "type")]
    pub value_type: Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    MultiPoint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Sync,
    Async,
}

#[derive(Debug, Deserialize)]
pub struct RouteParams {
    mode: Option<SyncMode>,
}

/// compute a route
async fn compute_route(
    mode: web::Query<RouteParams>,
    route: web::Json<RouteDefinition>,
) -> HttpResponse {
    dbg!(&mode.mode);
    dbg!(&route);
    let resp = json!({
      "type": "FeatureCollection",
      "name": "Route from A to B",
      "status": "successful",
      "links": [
        {
          "href": "https://example.com/routes/5e37f",
          "rel": "self",
          "type": "application/geo+json",
          "title": "this document"
        },
        {
          "href": "https://example.com/routes/5e37f/definition",
          "rel": "describedBy",
          "type": "application/json",
          "title": "route definition for this route"
        }
      ],
      "features": [
        {
          "type": "Feature",
          "id": 1,
          "geometry": {
            "type": "LineString",
            "coordinates": [
              [
                36.1234515,
                32.6453783
              ],
              [
                36.1230475,
                32.6474995
              ],
              [
                36.1226617,
                32.6496609
              ],
              [
                36.1222502,
                32.6517703
              ],
              [
                36.1218481,
                32.6539184
              ],
              [
                36.1214698,
                32.655952
              ],
              [
                36.121577,
                32.6581182
              ],
              [
                36.1217253,
                32.6602735
              ],
              [
                36.1218648,
                32.6625212
              ],
              [
                36.1221329,
                32.6670227
              ],
              [
                36.1222644,
                32.6693694
              ],
              [
                36.1223852,
                32.6713623
              ],
              [
                36.1225386,
                32.6737007
              ],
              [
                36.1226517,
                32.6758111
              ],
              [
                36.1227807,
                32.6780519
              ],
              [
                36.1229206,
                32.6803173
              ],
              [
                36.1230627,
                32.6826661
              ],
              [
                36.1231947,
                32.6848655
              ],
              [
                36.123323,
                32.6870967
              ],
              [
                36.1234537,
                32.6893521
              ],
              [
                36.1237022,
                32.693658
              ],
              [
                36.1238412,
                32.6957939
              ],
              [
                36.1239747,
                32.697965
              ],
              [
                36.1240873,
                32.7000159
              ],
              [
                36.1242391,
                32.7022258
              ],
              [
                36.1243499,
                32.7043422
              ],
              [
                36.1244689,
                32.7064962
              ],
              [
                36.124602,
                32.7085578
              ],
              [
                36.1247213,
                32.7106286
              ]
            ]
          },
          "properties": {
            "type": "route overview",
            "length_m": 1224.7,
            "duration_s": 75,
            "maxHeight_m": 4.5,
            "comment": "This is a place to add a comment about the processing of the route."
          }
        },
        {
          "type": "Feature",
          "id": 2,
          "geometry": {
            "type": "Point",
            "coordinates": [
              36.1234515,
              32.6453783
            ]
          },
          "properties": {
            "type": "start"
          }
        },
        {
          "type": "Feature",
          "id": 3,
          "geometry": {
            "type": "Point",
            "coordinates": [
              36.1214698,
              32.655952
            ]
          },
          "properties": {
            "type": "segment",
            "length_m": 123.2,
            "duration_s": 8,
            "instruction": "left",
            "roadName": "Main Street",
            "maxHeight_m": 4.5,
            "speedLimit": 35,
            "speedLimitUnit": "mph"
          }
        },
        {
          "type": "Feature",
          "id": 4,
          "geometry": {
            "type": "Point",
            "coordinates": [
              36.1247213,
              32.7106286
            ]
          },
          "properties": {
            "type": "segment",
            "length_m": 1101.5,
            "duration_s": 67,
            "roadName": "Chicago Avenue",
            "speedLimit": 50,
            "speedLimitUnit": "mph"
          }
        },
        {
          "type": "Feature",
          "id": 5,
          "geometry": {
            "type": "Point",
            "coordinates": [
              36.1247213,
              32.7106286
            ]
          },
          "properties": {
            "type": "end"
          }
        }
      ]
    });

    HttpResponse::Ok().json(resp)
}

#[cfg(feature = "ogcapi")]
pub fn init_service(api: &mut OgcApiInventory, openapi: &mut OpenApiDoc) {
    use bbox_common::ogcapi::ApiLink;

    api.landing_page_links.push(ApiLink {
        href: "/routes".to_string(),
        rel: Some("routes".to_string()),
        type_: Some("application/json".to_string()),
        title: Some("OGC API routes".to_string()),
        hreflang: None,
        length: None,
    });
    api.conformance_classes.extend(vec![
        // Core
        "http://www.opengis.net/spec/ogcapi-routes-1/1.0.0-draft.1/req/core".to_string(),
        // JSON
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/json".to_string(),
        // OpenAPI Specification
        "http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/oas30".to_string(),
        /*
         * OGC Process Description - http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/ogc-process-description
         * HTML - http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/html
         * Job list - http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/job-list
         * Callback - http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/callback
         * Dismiss - http://www.opengis.net/spec/ogcapi-processes-1/1.0/conf/dismiss
         */
    ]);
    openapi.extend(include_str!("openapi.yaml"), "/");
}

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/routes").route(web::post().to(compute_route)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body, dev::Service, http, test, App, Error};

    #[actix_web::test]
    async fn test_route() -> Result<(), Error> {
        let app = test::init_service(
            App::new().service(web::resource("/routes").route(web::post().to(compute_route))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/routes")
            .set_json(&RouteDefinition {
                name: Some("A to B".to_string()),
                preference: None,
                waypoints: Waypoints {
                    value_type: Type::MultiPoint,
                    coordinates: vec![vec![36.1234515, 32.6453783], vec![36.1234515, 32.6453783]],
                },
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = body::to_bytes(resp.into_body()).await?;

        assert_eq!(response_body, "{\"features\":[{\"geometry\":{\"coordinates\":[[36.1234515,32.6453783],[36.1230475,32.6474995],[36.1226617,32.6496609],[36.1222502,32.6517703],[36.1218481,32.6539184],[36.1214698,32.655952],[36.121577,32.6581182],[36.1217253,32.6602735],[36.1218648,32.6625212],[36.1221329,32.6670227],[36.1222644,32.6693694],[36.1223852,32.6713623],[36.1225386,32.6737007],[36.1226517,32.6758111],[36.1227807,32.6780519],[36.1229206,32.6803173],[36.1230627,32.6826661],[36.1231947,32.6848655],[36.123323,32.6870967],[36.1234537,32.6893521],[36.1237022,32.693658],[36.1238412,32.6957939],[36.1239747,32.697965],[36.1240873,32.7000159],[36.1242391,32.7022258],[36.1243499,32.7043422],[36.1244689,32.7064962],[36.124602,32.7085578],[36.1247213,32.7106286]],\"type\":\"LineString\"},\"id\":1,\"properties\":{\"comment\":\"This is a place to add a comment about the processing of the route.\",\"duration_s\":75,\"length_m\":1224.7,\"maxHeight_m\":4.5,\"type\":\"route overview\"},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[36.1234515,32.6453783],\"type\":\"Point\"},\"id\":2,\"properties\":{\"type\":\"start\"},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[36.1214698,32.655952],\"type\":\"Point\"},\"id\":3,\"properties\":{\"duration_s\":8,\"instruction\":\"left\",\"length_m\":123.2,\"maxHeight_m\":4.5,\"roadName\":\"Main Street\",\"speedLimit\":35,\"speedLimitUnit\":\"mph\",\"type\":\"segment\"},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[36.1247213,32.7106286],\"type\":\"Point\"},\"id\":4,\"properties\":{\"duration_s\":67,\"length_m\":1101.5,\"roadName\":\"Chicago Avenue\",\"speedLimit\":50,\"speedLimitUnit\":\"mph\",\"type\":\"segment\"},\"type\":\"Feature\"},{\"geometry\":{\"coordinates\":[36.1247213,32.7106286],\"type\":\"Point\"},\"id\":5,\"properties\":{\"type\":\"end\"},\"type\":\"Feature\"}],\"links\":[{\"href\":\"https://example.com/routes/5e37f\",\"rel\":\"self\",\"title\":\"this document\",\"type\":\"application/geo+json\"},{\"href\":\"https://example.com/routes/5e37f/definition\",\"rel\":\"describedBy\",\"title\":\"route definition for this route\",\"type\":\"application/json\"}],\"name\":\"Route from A to B\",\"status\":\"successful\",\"type\":\"FeatureCollection\"}");

        Ok(())
    }
}

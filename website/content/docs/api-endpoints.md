---
weight: 5
---

# API Endpoints

## OGC API Endpoints

Services are available via the HTTP `GET` endpoints:

|                  URL                   |                  Description                   |
|----------------------------------------|------------------------------------------------|
| `/`                                    | Landing page (HTML or JSON)                    |
| `/conformance`                         | API conforomance                               |
| `/openapi`                             | OpenAPI specification (YAML)                   |
| `/openapi.yaml`                        | OpenAPI specification (YAML)                   |
| `/openapi.json`                        | OpenAPI specification (JSON)                   |


Available formats:

|   URL   |        Description        |
|---------|---------------------------|
| `.json` | JSON / GeoJSON format     |
| `.html` | HTML format, if available |


## BBOX API Endpoints


|    URL    |     Description     |
|-----------|---------------------|
| `/health` | Server health check |


## Request examples

    curl -s -H 'Accept: application/json' http://localhost:8080/ | jq .

    curl -s http://localhost:8080/openapi.json | jq .

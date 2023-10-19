BBOX OGC API Features Service
=============================

Asynchronous OGC API Features server implementation.

Features:
- [ ] OGC API - Features - Part 1: Core 1.0
- [ ] OGC API - Features - Part 2: Coordinate Reference Systems by Reference 1.0
- [ ] OpenAPI endpoint
- [ ] GeoZero backend (PostGIS, GeoPackage, ...)


## Usage

    cargo run serve

    x-www-browser http://127.0.0.1:8080/

Built-in Swagger UI:

http://localhost:8080/openapi/

View API in external Swagger editor:

https://editor.swagger.io/?url=http://localhost:8080/api

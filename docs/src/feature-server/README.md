# BBOX OGC API Features Service

Asynchronous OGC API Features server implementation.

Features:
- [x] OGC API - Features - Part 1: Core 1.0
- [ ] OGC API - Features - Part 2: Coordinate Reference Systems by Reference 1.0
- [x] OpenAPI endpoint
- [x] Builtin storage backends: PostGIS, GeoPackage
- [x] Output formats: GeoJSON


## Usage

Run feature server with `bbox.toml` configuration:

    bbox-feature-server serve

or with a custom configuration:

    bbox-feature-server --config=bbox-pg.toml serve

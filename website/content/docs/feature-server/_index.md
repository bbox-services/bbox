---
weight: 6
next: docs/feature-server/configuration
---

# BBOX OGC API Features Service

Asynchronous OGC API Features server implementation.

Features:
- [x] OGC API - Features - Part 1: Core 1.0
- [ ] OGC API - Features - Part 2: Coordinate Reference Systems by Reference 1.0
- [x] Builtin storage backends: PostGIS, GeoPackage
- [x] SQL queries with time and custom query parameters 
- [x] Output formats: GeoJSON
- [x] Compatibility: WFS + WFS-T via QGIS Server


## Usage

Run feature server with `bbox.toml` configuration:

    bbox-feature-server serve

or with a custom configuration:

    bbox-feature-server --config=bbox-pg.toml serve

# BBOX OGC API Features Service

Asynchronous OGC API Features server implementation.

Features:
- [x] OGC API - Features - Part 1: Core 1.0
- [ ] OGC API - Features - Part 2: Coordinate Reference Systems by Reference 1.0
- [x] OpenAPI endpoint
- [x] Native backends: PostGIS, GeoPackage
- [x] Output formats: GeoJSON


## Configuration

Datasources:
```toml
[[datasource]]
name = "t_rex_tests"
[datasource.postgis]
url = "postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests"

[[datasource]]
name = "ne_extracts"
[datasource.gpkg]
path = "../assets/ne_extracts.gpkg"
```

Collections with auto detection:
```toml
[[collections.postgis]]
url = "postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests"

[[collections.directory]]
dir = "../assets"
```

Collections:
```toml
[[collection]]
name = "populated_places"
title = "populated places"
description = "Natural Earth populated places"
[collection.gpkg]
datasource = "ne_extracts"
table = "ne_10m_populated_places"
```

## Usage

Run feature server with bbox.toml configuration:

    cargo run serve


Inspect collections:

    x-www-browser http://127.0.0.1:8080/collections

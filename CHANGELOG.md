# Changelog

## 0.5.0 alpha5 (2024-03-11)

* Add basic CORS header support
* Support MBTiles as tile datasource and tile store
* Add MBTiles metadata.json service endpoint
* Support PMTiles as tile datasource and tile store
* Support different fields in zoom-dependent layer queries
* T-rex configuration support
* Add basic diagnostics tile layer
* Fix PG queries with reprojected layers
* CLI args for tile stores and tile datasources
* Fix map request trace spans
* Support temporal and queryable fields for Postgis collections
* FCGI auto-detection on Fedora

## 0.5.0 alpha4 (2023-10-30)

* Use ST_AsMvtGeom for PostGIS tilesets
* Tileset configuration: Rename wms_project to map_service
* Various tile server and feature server fixes and improvements
* Update embedded Maplibre & OL

## 0.5.0 alpha3 (2023-10-21)

* Unify collection and tileserver datasource configuration
* Add configurable collections with optional custom SQL
* Support tile cache seeding with embedded map servers

## 0.5.0 alpha2 (2023-09-11)

* Add PostGIS tile source
* Use cache from config in seeder
* Make tile cache overwriting optional
* Add loglevel command line argument
* Add PostGIS routing datasource

## 0.5.0 alpha1 (2023-07-06)

* Fix collection list
* Switch frontend to DaisyUI / Tailwind CSS
* Embed JS + CSS for OL, Maplibre, Proj, Redoc and SwaggerUI
* Enable HTML frontend in Windows build

## 0.5.0 alpha0 (2023-06-26)

First public release.

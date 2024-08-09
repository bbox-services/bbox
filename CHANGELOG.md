# Changelog

## 0.6.0 (2024-08-09)

* Use config file path as base directory
* Linux ARM binaries
* Publication on crates.io

## 0.6.0 beta1 (2024-08-08)

* Support tile grids with up to 255 levels
* Add queryables endpoint to feature server
* Return layer min-/maxzoom in Tilejson
* Update to tile-grid 0.6 / ogcapi-types 0.2
* Support multiple grids per tileset with opt. zoom limits
* Add cache control max-age config with opt. zoom levels
* Detect MBTiles layout for updating
* Use exe_location configuration for map backends
* CI improvements (PostGIS tests, etc.)
* Guides and other doc improvements

## 0.5.0 (2024-04-03)

No changes since 0.5.0 beta4

## 0.5.0 beta4 (2024-04-03)

* Detect compression in MBTiles
* Fix landing page without map server feature
* Return available tilesets in /tiles endpoint
* Don't open QWC2 viewer with inactive feature
* Emit error message for missing map service feature

## 0.5.0 beta3 (2024-04-02)

* Support reading/writing compressed tiles from/into cache
* Add compression config for file and S3 tile stores
* Add env var for PostGIS datasource

## 0.5.0 beta2 (2024-03-29)

* Support simplification in ST_AsMvt mode
* Add loglevel to webserver config
* Disable QWC2 viewer in default build
* Publish deb packages

## 0.5.0 beta1 (2024-03-18)

* Add x, y and custom fields to layer queries
* Keep order of PG tile layers
* Add bbox-tile-server release assets

## 0.5.0 alpha6 (2024-03-14)

* Fix startup failure without CORS configuration

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

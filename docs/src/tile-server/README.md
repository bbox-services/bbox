# BBOX tile server

Map tile serving and tile cache seeding.

Features:
- [x] OGC API – Tiles - Part 1: Core 1.0
- [x] Vector tile server
  - [X] Vector data source: PostGIS
  - [X] Tile archives: MBTiles, PMTiles
- [x] Raster tile server (Backends: QGIS Server and MapServer)
- [x] Tile proxy server (WMS backend)
- [x] XYZ tile service endpoint with TileJSON metadata
- [x] Support for Custom Tile Matrix Sets
- [ ] OGC WMTS (via map service backend)

Tile seeder features:
- [x] Parallelized seeding of raster and vector tiles
- [x] Storage backends: Files, S3, MBTiles, PMTiles


## Usage

Run tile server with `bbox.toml` configuration:

    bbox-tile-server serve

Run tile server with auto discovery:

    bbox-tile-server serve ../assets/liechtenstein.mbtiles

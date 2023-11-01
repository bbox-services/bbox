# BBOX tile server

Map tile serving and tile cache seeding.

Features:
- [x] Vector tile server (Source: PostGIS)
- [x] Raster tile server (Backends: QGIS Server and MapServer)
- [x] Tile proxy server (Backends: WMS)
- [x] OGC API - Tiles Core
- [x] XYZ vector tiles with TileJSON metadata
- [ ] OGC WMTS (via map service backend)


Tile seeder features:
- [x] Parallelized seeding of raster and vector tiles
- [x] Storage Backend: S3 - optimized for tile transfer
- [x] Storage Backend: Local files


## Usage

Run tile server with `bbox.toml` configuration:

    bbox-tile-server serve

Run tile server with auto discovery:

    bbox-tile-server serve ../assets/liechtenstein.mbtiles

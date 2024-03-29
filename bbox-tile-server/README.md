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


## Configuration

See [Documentation](https://www.bbox.earth/tile-server/configuration.html) for examples.

## Usage

Run tile server with bbox.toml configuration:

    # Reduce log output for testing
    export BBOX_MAPSERVER__NUM_FCGI_PROCESSES=1
    cargo run serve

Tile requests:

    curl -o /tmp/tile.png http://localhost:8080/xyz/ne_extracts/2/2/2.png

    curl -o /tmp/tile.png http://localhost:8080/xyz/ne_umn/2/2/2.png

    curl -o /tmp/tile.jpg http://localhost:8080/xyz/gebco/0/0/0.jpeg

    curl -o /tmp/tile.mvt http://localhost:8080/xyz/mbtiles_mvt_fl/14/8621/5759.mvt

    curl -o /tmp/tilegz.mvt -H 'Content-Encoding: gzip' http://localhost:8080/xyz/mbtiles_mvt_fl/14/8621/5759.mvt

    curl -o /tmp/tile.png -H 'Accept: image/png; mode=8bit' http://localhost:8080/map/tiles/ne_extracts/2/2/2

Run tile server without configuration:

    cargo run -- serve ../assets/liechtenstein.mbtiles

    curl -o /tmp/tile.mvt http://localhost:8080/xyz/liechtenstein/14/8621/5759.mvt

OGC API entry points:

    curl -s -H 'Accept: application/json' http://localhost:8080/ | jq .

    curl -s http://localhost:8080/openapi.json | jq .

XYZ URL (Leaflet, QGIS, etc.):

    http://localhost:8080/xyz/ne_extracts/{z}/{x}/{y}.png

Tilejson requests:

    curl -s http://localhost:8080/xyz/mbtiles_mvt_fl.json | jq .

Style JSON requests:

    curl -s http://localhost:8080/xyz/mbtiles_mvt_fl.style.json | jq .

    curl -s http://localhost:8080/xyz/ne_extracts.style.json | jq .

Metadata requests:

    curl -s http://localhost:8080/xyz/mbtiles_mvt_fl/metadata.json | jq .

Map viewer examples:

    x-www-browser http://127.0.0.1:8080/assets/usergrid.html?debug=1

Map viewer template examples:

    x-www-browser http://localhost:8080/html/maplibre/mbtiles_mvt_fl?style=/assets/mbtiles_mvt_fl-style.json

With PostGIS Service:

    just start-db
    just serve

    curl -s http://localhost:8080/xyz/ne_countries.style.json | jq .

    x-www-browser http://localhost:8080/assets/maplibre.html?style=/xyz/ne_countries.style.json

Tile request:

    curl -o /tmp/tile.mvt http://localhost:8080/xyz/ne_countries/2/2/2.pbf

With filter parameter:

    curl -o /tmp/tile.mvt 'http://localhost:8080/xyz/taxi_zones/0/0/0.mvt?color=3.0'


### Seeding

Relase Build:

    cargo build --release

Seed with proxy WMS:

    ../target/release/bbox-tile-server seed --tileset=gebco --tile-path=/tmp/tiles/gebco --maxzoom=2

Seed with embedded map service:

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --tile-path=/tmp/tiles/ne_extracts --maxzoom=2

Seed PostGIS MVT tiles:

    ../target/release/bbox-tile-server seed --tileset=ne_countries --tile-path=/tmp/tiles/ne_countries --maxzoom=2

#### Seed to S3 storage

See [performance.md](./performance.md) for a local S3 test setup.

Set S3 env vars:

    export S3_ENDPOINT_URL="http://localhost:9000"
    export AWS_ACCESS_KEY_ID=miniostorage
    export AWS_SECRET_ACCESS_KEY=miniostorage

Seed raster tiles:

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --s3-path=s3://tiles --maxzoom=5


### Using Maputnik for MVT styling

* Download latest public.zip from https://github.com/maputnik/editor/releases
* Unpack into ../assets/ and rename public to maputnik

Open example:

    http://localhost:8080/assets/maputnik/index.html?style=http://localhost:8080/assets/maplibre-style.json
    http://localhost:8080/assets/maputnik/index.html#11.0/47.0944/9.5076

    http://localhost:8080/assets/maputnik/index.html?style=http://localhost:8080/xyz/mbtiles_mvt_fl.style.json

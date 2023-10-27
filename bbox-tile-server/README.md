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

Datasources:
```toml
[[datasource]]
name = "t_rex_tests"
#default = true
[datasource.postgis]
url = "postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests"

[[datasource]]
name = "ne_extracts"
[datasource.gpkg]
path = "../assets/ne_extracts.gpkg"

[[datasource]]
name = "gebco"
[datasource.wms_proxy]
baseurl = "https://www.gebco.net/data_and_products/gebco_web_services/web_map_service/mapserv?version=1.3.0"
format = "image/jpeg"
```

Vector tiles from PostGIS table:
```toml
[[tileset]]
name = "ne_10m_populated_places"
[tileset.postgis]
datasource = "t_rex_tests"
extent = [-179.58998, -90.00000, 179.38330, 82.48332]
[[tileset.postgis.layer]]
name = "ne_10m_populated_places"
table_name = "ne.ne_10m_populated_places"
#geometry_field = "wkb_geometry"
fid_field = "fid"
geometry_type = "POINT"
srid = 3857
buffer_size = 0
#make_valid = true
query_limit = 1000
#[[tileset.postgis.layer.query]]
#sql = """SELECT wkb_geometry,fid,scalerank,name,pop_max FROM ne.ne_10m_populated_places"""
```

Raster tiles with QGIS Server backend:
```toml
[[tileset]]
name = "ne_extracts"
map_service = { project = "ne_extracts", suffix = "qgz", layers = "ne_extracts" }
cache = "tilecache"
```

Raster tiles with UMN Mapserver backend:
```toml
[[tileset]]
name = "ne_umn"
map_service = { project = "ne", suffix = "map", layers = "country", tile_size = 512 }
```

Raster tiles from external WMS:
```toml
[[tileset]]
name = "gebco"
wms_proxy = { source = "gebco", layers = "gebco_latest" }
```

Tile caches:
```toml
[[tilecache]]
name = "tilecache"
[tilecache.files]
base_dir = "/tmp/tilecache"

[[tilecache]]
name = "aws"
[tilecache.s3]
path = "s3://tiles"
```

Custom tile grid:
```toml
[[grid]]
json = "../assets/custom-grid-lv95.json"
```


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

Map viewer examples:

    x-www-browser http://127.0.0.1:8080/assets/usergrid.html?debug=1

Map viewer template examples:

    x-www-browser http://localhost:8080/html/maplibre-asset-style/mbtiles_mvt_fl

With PG Service:

    curl -s http://localhost:8080/xyz/ne_10m_populated_places.style.json | jq .
    x-www-browser http://localhost:8080/html/maplibre/ne_10m_populated_places


### Seeding

Relase Build:

    cargo build --release

Seed with proxy WMS:

    ../target/release/bbox-tile-server seed --tileset=gebco --base-dir=/tmp/tiles/gebco --maxzoom=2

Seed with embedded map service:

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --base-dir=/tmp/tiles/ne_extracts --maxzoom=2

Seed PostGIS MVT tiles:

    ../target/release/bbox-tile-server seed --tileset=ne_10m_populated_places --base-dir=/tmp/tiles/ne_10m_populated_places --maxzoom=2

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

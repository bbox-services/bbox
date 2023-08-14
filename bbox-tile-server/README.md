BBOX tile server
================

Map tile serving and tile cache seeding.

Features:
- [x] Vector tile server (Source: PostGIS)
- [x] Raster tile server (Backends: QGIS Server and MapServer)
- [x] Tile proxy server (Backends: WMS)
- [x] OGC API - Tiles Core
- [x] XYZ vector tiles with TileJSON metadata
- [ ] OGC WMTS (via map service backend)


Tile seeder features:
- [ ] Parallelized seeding of raster and vector tiles
- [x] Storage Backend: S3 - optimized for tile transfer
- [x] Storage Backend: Local files


## Configuration

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
wms_project = { project = "ne_extracts", suffix = "qgz", layers = "ne_extracts" }
cache = "tilecache"
```

Raster tiles with UMN Mapserver backend:
```toml
[[tileset]]
name = "ne_umn"
wms_project = { project = "ne", suffix = "map", layers = "country", tile_size = 512 }
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


Relase Build:

    cargo build --release

Local file seeding test:

    ../target/release/bbox-tile-server seed --tileset=gebco --base-dir=/tmp/tiles --maxzoom=2

Set S3 env vars:

    export S3_ENDPOINT_URL="http://localhost:9000"
    export AWS_ACCESS_KEY_ID=miniostorage
    export AWS_SECRET_ACCESS_KEY=miniostorage

Run:

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --s3-path=s3://tiles --maxzoom=5


### Local S3 tests

Install [MinIO Client](https://github.com/minio/mc):

    cd /usr/local/bin
    wget https://dl.min.io/client/mc/release/linux-amd64/mc
    chmod +x mc
    mc --help

Setup storage directory:

    mkdir s3data

Run MinIO:

    docker run --security-opt label=disable -d --rm --name minio -p 9000:9000 -p 9001:9001 -v $PWD/s3data:/data -e MINIO_REGION_NAME=my-region -e MINIO_ROOT_USER=miniostorage -e MINIO_ROOT_PASSWORD=miniostorage minio/minio server /data --console-address ":9001"

Setup Bucket:

    export AWS_ACCESS_KEY_ID=miniostorage
    export AWS_SECRET_ACCESS_KEY=miniostorage

    mc config host add local-docker http://localhost:9000 miniostorage miniostorage
    mc mb local-docker/tiles
    mc policy set public local-docker/tiles

Access MinIO Console: http://localhost:9001

Stop MinIO:

    docker stop minio


### Using Maputnik for MVT styling

* Download latest public.zip from https://github.com/maputnik/editor/releases
* Unpack into ../assets/ and rename public to maputnik

Open example:

    http://localhost:8080/assets/maputnik/index.html?style=http://localhost:8080/assets/maplibre-style.json
    http://localhost:8080/assets/maputnik/index.html#11.0/47.0944/9.5076

    http://localhost:8080/assets/maputnik/index.html?style=http://localhost:8080/xyz/mbtiles_mvt_fl.style.json


### S3 upload benchmarks

#### s3cmd

    time s3cmd sync ~/code/gis/vogeldatenbank/tiles/ s3://tiles

    Done. Uploaded 448854168 bytes in 95.8 seconds, 4.47 MB/s.

    -> real    1m38.220s

#### s5cmd

    export S3_ENDPOINT_URL="http://localhost:9000"

    time s5cmd cp /home/pi/code/gis/vogeldatenbank/tiles/ s3://tiles

    -> real    0m15.807s

    time s5cmd rm s3://tiles/*

    -> real    0m3.856s

#### bbox-tile-server

Initial sequential implementation:

    export S3_ENDPOINT_URL="http://localhost:9000"

    cargo build --release
    time ../target/release/bbox-tile-server upload --srcdir=/home/pi/code/gis/vogeldatenbank/tiles/ --s3-path=s3://tiles

    -> real    0m53.257s

Parallel tasks:

    Default values (8+2 threads / 256 tasks)

    -> real    0m13.578s (10s-20s)

#### WMS requests

Local QGIS NaturalEarth WMS

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --maxzoom=18 --s3-path=s3://tiles

    -> 14s

Local QGIS NaturalEarth WMS to local directory

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --maxzoom=18 --base-dir=/tmp/tiles

    -> 13s

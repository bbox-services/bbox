# BBOX tile server performance measurements

## Local S3 test setup

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


## Seeding tests

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


## MVT benchmark

https://github.com/pka/mvt-benchmark

Start DB:

    just start

Local file seeding benchmark:

    just seed-files

Results:

* bbox-tile-server v0.5.0-alpha4: 6.5s


## S3 upload benchmarks

### s3cmd

    time s3cmd sync ~/code/gis/vogeldatenbank/tiles/ s3://tiles

    Done. Uploaded 448854168 bytes in 95.8 seconds, 4.47 MB/s.

    -> real    1m38.220s

### s5cmd

    export S3_ENDPOINT_URL="http://localhost:9000"

    time s5cmd cp /home/pi/code/gis/vogeldatenbank/tiles/ s3://tiles

    -> real    0m15.807s

    time s5cmd rm s3://tiles/*

    -> real    0m3.856s

### bbox-tile-server

Initial sequential implementation:

    export S3_ENDPOINT_URL="http://localhost:9000"

    cargo build --release
    time ../target/release/bbox-tile-server upload --srcdir=/home/pi/code/gis/vogeldatenbank/tiles/ --s3-path=s3://tiles

    -> real    0m53.257s

Parallel tasks:

    Default values (8+2 threads / 256 tasks)

    -> real    0m13.578s (10s-20s)

### WMS requests

Local QGIS NaturalEarth WMS

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --maxzoom=18 --s3-path=s3://tiles

    -> 14s

Local QGIS NaturalEarth WMS to local directory

    ../target/release/bbox-tile-server seed --tileset=ne_extracts --maxzoom=18 --base-dir=/tmp/tiles

    -> 13s

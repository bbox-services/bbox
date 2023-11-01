# Tile seeding

Seed with proxy WMS:

    bbox-tile-server seed --tileset=gebco --base-dir=/tmp/tiles/gebco --maxzoom=2

Seed with embedded map service:

    bbox-tile-server seed --tileset=ne_extracts --base-dir=/tmp/tiles/ne_extracts --maxzoom=2

Seed PostGIS MVT tiles:

    bbox-tile-server seed --tileset=ne_countries --base-dir=/tmp/tiles/ne_countries --maxzoom=2

## Seed to S3 storage

Set S3 env vars:

    export S3_ENDPOINT_URL="http://localhost:9000"
    export AWS_ACCESS_KEY_ID=miniostorage
    export AWS_SECRET_ACCESS_KEY=miniostorage

Seed raster tiles:

    bbox-tile-server seed --tileset=ne_extracts --s3-path=s3://tiles --maxzoom=5

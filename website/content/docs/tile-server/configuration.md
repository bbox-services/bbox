---
weight: 2
---

# Tile server configuration

## Datasources

```toml
[[datasource]]
name = "mvtbenchdb"
[datasource.postgis]
url = "postgresql://mvtbench:mvtbench@127.0.0.1:5439/mvtbench"

[[datasource]]
name = "gebco"
[datasource.wms_proxy]
baseurl = "https://www.gebco.net/data_and_products/gebco_web_services/web_map_service/mapserv?version=1.3.0"
format = "image/jpeg"
```

## Vector tiles from PostGIS table

```toml
[[tileset]]
name = "ne_countries"
[tileset.postgis]
datasource = "mvtbenchdb"
extent = [-179.97277, -83.05457, 179.99366, 83.23559]
attribution = "Natural Earth v4"

[[tileset.postgis.layer]]
name = "country-name"
#table_name = "ne_10m_admin_0_country_points"
geometry_type = "POINT"
[[tileset.postgis.layer.query]]
sql = """SELECT wkb_geometry, abbrev, name FROM ne_10m_admin_0_country_points"""
```

Query with a custom runtime parameter:
```toml
sql = """
SELECT id, date, ST_Point(lon, lat, 4326) AS geom
FROM gpslog
WHERE date = !date!
"""
```

A custom parameter is passed by name: `/xyz/gpstracks/0/0/0.mvt?date=2024-11-08`


## Raster tiles from map service

QGIS Server backend:
```toml
[[tileset]]
name = "ne_extracts"
map_service = { project = "ne_extracts", suffix = "qgz", layers = "ne_extracts" }
cache = "tilecache"
```

UMN Mapserver backend:
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

## Tile caches

```toml
[[tilecache]]
name = "tilecache"
[tilecache.files]
base_dir = "/tmp/tilecache"

[[tilecache]]
name = "aws"
[tilecache.s3]
path = "s3://tiles"

[[tilestore]]
name = "mbtilecache"
[tilestore.mbtiles]
path = "/tmp/tilecache.mbtiles"

[[tilestore]]
name = "pmtilecache"
[tilestore.pmtiles]
path = "/tmp/tilecache.pmtiles"
```

To use a tilecache when serving tiles, add the tilecache name to the tileset:

```toml
[[tileset]]
name = "ne_countries"
cache = "tilecache"
```

## Custom tile grid

```toml
[[grid]]
json = "assets/custom-grid-lv95.json"
```

To use the custom tile grid, add the tms name to the tileset:

```toml
[[tileset]]
name = "rivers_lakes"
[[tileset.tms]]
id = "LV95"
```

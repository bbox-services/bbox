# Feature server configuration

## Datasources

```toml
[[datasource]]
name = "mvtbenchdb"
[datasource.postgis]
url = "postgresql://mvtbench:mvtbench@127.0.0.1:5439/mvtbench"

[[datasource]]
name = "ne_extracts"
[datasource.gpkg]
path = "../data/ne_extracts.gpkg"
```

## Collections with auto discovery

```toml
[[collections.postgis]]
url = "postgresql://mvtbench:mvtbench@127.0.0.1:5439/mvtbench"

[[collections.directory]]
dir = "../data"
```

## Collections

```toml
[[collection]]
name = "populated_places"
title = "populated places"
description = "Natural Earth populated places"
[collection.gpkg]
datasource = "ne_extracts"
table = "ne_10m_populated_places"
```

With custom SQL query:
```toml
[[collection]]
name = "populated_places_names"
title = "populated places names"
description = "Natural Earth populated places"
[collection.gpkg]
datasource = "ne_extracts"
sql = "SELECT fid, name, geom FROM ne_10m_populated_places"
geometry_field = "geom"
fid_field = "fid"
```

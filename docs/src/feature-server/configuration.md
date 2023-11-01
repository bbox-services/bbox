# Feature server configuration

## Datasources

```toml
[[datasource]]
name = "t_rex_tests"
[datasource.postgis]
url = "postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests"

[[datasource]]
name = "ne_extracts"
[datasource.gpkg]
path = "../assets/ne_extracts.gpkg"
```

## Collections with auto discovery

```toml
[[collections.postgis]]
url = "postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests"

[[collections.directory]]
dir = "../assets"
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

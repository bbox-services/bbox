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
dir = "../data" # Relative to configuration file
```

## Collections

```toml
[[collection]]
name = "populated_places"
title = "populated places"
description = "Natural Earth populated places"
[collection.gpkg]
datasource = "ne_extracts"
table_name = "ne_10m_populated_places"
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

Collections with a PostGIS datasource:
```toml
[[collection]]
name = "states_provinces_lines"
title = "States/provinces borders"
description = "Natural Earth states/provinces borders"
[collection.postgis]
datasource = "mvtbenchdb"
table_name = "ne_10m_admin_1_states_provinces_lines"

[[collection]]
name = "country_labels"
title = "Country names"
description = "Natural Earth country names"
[collection.postgis]
datasource = "mvtbenchdb"
sql = "SELECT fid, abbrev, name, wkb_geometry FROM ne_10m_admin_0_country_points"
geometry_field = "wkb_geometry"
fid_field = "fid"
```

With queriable fields:
```toml
[[collection]]
name = "gpstracks"
title = "GPS tracks"
description = "Daily GPS tracks"
[collection.postgis]
datasource = "trackingdb"
sql = "SELECT id, date, ST_Point(lon, lat, 4326) AS geom FROM gpslog"
geometry_field = "geom"
fid_field = "id"
queryable_fields = ["date"]
```

Queriable fields are passed by name: `/collections/gpstracks/items?date=2024-11-08`

Temporal filters can be applied by configuring `temporal_field` and optionally `temporal_end_field`.

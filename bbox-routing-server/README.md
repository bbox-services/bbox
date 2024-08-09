# BBOX Routing Service

Routing services with Contraction Hierarchy.

Features:
- [ ] OGC API - Routes - Part 1: Core
- [x] Multiple search APIs
  - [x] OGC API route requests
  - [x] Basic from/to requests
  - [x] Valhalla API compatible requests
- [x] Builtin storage backends: PostGIS, GeoPackage
- [ ] Extract routing graphs from OSM planet files


## Configuration

GeoPackage line geometry table:
```toml
[[routing.service]]
profile = "railway"
gpkg = "assets/railway-test.gpkg"
table = "flows"
geom = "geom"
```

PostGIS Edge/Vertices tables:
```toml
# Node search distance
search_dist = 0.01
# Edge table
table = "rail_arcs"
# Node/Vertices table
node_table = "rail_arcs_vertices_pgr"
# Geometry column
geom = "geom"
# Node ID column in node table
node_id = "id"
# Cost column
cost = "cost"
# Column with source node ID
node_src = "source"
# Column with destination (target) node ID
node_dst = "target"
```

This assumes tables created e.g. with PgRouting `pgr_createTopology`.

The contraction hierarchy is created on first startup and stored as cache files named `.graph.bin` and  `.nodes.bin`


Usage
-----

Request examples:

    curl -s -X 'POST' \
      'http://localhost:8080/routes?mode=sync' \
      -H 'accept: application/geo+json' \
      -H 'Content-Type: application/json' \
      -d '{
      "name": "Route from A to B",
      "waypoints": {
        "type": "MultiPoint",
        "coordinates": [
          [9.35213353, 47.0935012],
          [9.3422712, 47.1011887]
        ]
      },
      "preference": "fastest",
      "dataset": "OSM"
    }'

Basic from/to request:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=9.35213353,47.0935012&to_pos=9.3422712,47.1011887'

Zurich - Munich:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=8.53636,47.37726&to_pos=11.56096,48.14019'


Valhalla endpoint (e.g. for Valhalla QGIS Plugin):

Base URL: http://localhost:8080/routes/valhalla

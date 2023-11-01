# Routing Service Configuration

## GeoPackage line geometry table

```toml
[[routing.service]]
profile = "railway"
gpkg = "../assets/railway-test.gpkg"
table = "flows"
geom = "geom"
```

## PostGIS Edge/Vertices tables

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

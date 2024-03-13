# Routing Server Reference

* `service[]` [RoutingService](#routingservice)

## RoutingService

Routing service configuration
* `profile` *String* (optional)
* `search_dist` *f64* (optional): Node search distance
* `gpkg` *String*
* `postgis` [DsPostgis](#dspostgis) (optional)
* `table` *String*: Edge table
* `node_table` *String* (optional): Node/Vertices table
* `geom` *String*: Geometry column
* `node_id` *String* (optional): Node ID column in node table
* `cost` *String* (optional): Cost column (default: geodesic line length)
* `node_src` *String* (optional): Column with source node ID
* `node_dst` *String* (optional): Column with destination (target) node ID

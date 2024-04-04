# Guides

## Migration from t-rex

BBOX tile server has the same concepts as t-rex. Tile services are organized in tilesets containing layers with
database tables or zoom level dependent SQL queries.

Major differences:
- BBOX provides OGC API Tiles endpoints
- BBOX has more tile source types, including raster tiles produced by the included map service
- BBOX uses the PostGIS `ST_AsMvt` function, but also provides t-rex compatible queries
- t-rex supports GDAL vector sources for tiles, from which currently only GeoPackages are supported by BBOX
- t-rex has PostGIS connections on layer level, BBOX has one common source per tileset
- t-rex supports autodetection of database tables with optional configuration file generation

### Using the `--t-rex-config` command line option

`bbox-tile-server serve` supports reading t-rex configuration files. These are converted to
a BBOX configuration and printed in the log output. Most features can be automatically
translated, for others a warning message is emitted.

### Compatibility options

In case the PostGIS `ST_AsMvt` function produces unwanted results, setting `postgis2` to `true` activates
t-rex compatible SQL queries. Other layer options like `simplify`, `tolerance`, `make_valid` and `shift_longitude` will
then have exactly the same effect as in t-rex.

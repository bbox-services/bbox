# Guides

## Creating a custom grid

BBOX expects grid definitions according to the OGC Two Dimensional Tile Matrix Set specification found in https://docs.ogc.org/is/17-083r4/17-083r4.html.

Some reference grids are included in [tile-grid](https://github.com/pka/tile-grid/tree/main/data).

If you can't find an official grid for your need, you can use [morecantile](https://developmentseed.org/morecantile/) for creating a quadratic custom grid.

First you have to know the spatial reference of your grid. Find it e.g. on <https://spatialreference.org/>

Then you have to know the corners of your grid in the grid projection. If you know them in an other reference system, you can use the Proj cli tool [cs2cs](https://proj.org/en/9.4/apps/cs2cs.html)
to transform the known points.

Example:
```
echo -90 -180 | cs2cs EPSG:4326 EPSG:8857
-10216474.79    -8392927.60 0.00
```

With this parameters, you can create a JSON file with the tile grid description.

Example:
```
morecantile custom --epsg 8857 --extent -17243959.06 -8392927.6 17243959.06 8392927.6 --name "EqualEarthGreenwichWGS84Rect" --title "Equal Earth Greenwich WGS84" | jq . >EqualEarthGreenwichWGS84Rect.json
```

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

To create an OGC conformant grid definition, follow the instructions on [Creating a custom grid](#creating-a-custom-grid).

### Compatibility options

In case the PostGIS `ST_AsMvt` function produces unwanted results, setting `postgis2` to `true` activates
t-rex compatible SQL queries. Other layer options like `simplify`, `tolerance`, `make_valid` and `shift_longitude` will
then have exactly the same effect as in t-rex.

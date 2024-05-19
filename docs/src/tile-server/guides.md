# Guides

## Creating a custom grid

BBOX expects grid definitions according to the OGC Two Dimensional Tile Matrix Set specification found in <https://docs.ogc.org/is/17-083r4/17-083r4.html>.

Some reference grids are included in [tile-grid](https://github.com/pka/tile-grid/tree/main/data).

If you can't find an official grid for your need, you can use [morecantile](https://developmentseed.org/morecantile/) for creating a quadratic custom grid.

First you have to know the spatial reference of your grid. Find it e.g. on <https://spatialreference.org/>

Then you have to know the corners of your grid in the grid projection. If you know them in an other reference system, you can use the Proj cli tool [cs2cs](https://proj.org/en/9.4/apps/cs2cs.html)
to transform the known points.

Example:
```
echo -90 -180 | cs2cs EPSG:4326 EPSG:8857
-10216474.79    -8392927.60 0.00
echo 0 -180 | cs2cs EPSG:4326 EPSG:8857
-17243959.06    0.00 0.00
```

With this parameters, you can create a JSON file with the tile grid description.

Example:
```
morecantile custom --epsg 8857 --extent -17243959.06 -17243959.06 17243959.06 17243959.06 --name "EqualEarthGreenwichWGS84Quad" --title "Equal Earth Greenwich WGS84" | jq . >EqualEarthGreenwichWGS84Quad.json
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


## Troubleshooting

### Inspect tiles with GDAL

You can use GDAL with the [MVT driver](https://gdal.org/drivers/vector/mvt.html) to inspect and convert MVT tiles.

Inspect with `ogrinfo`:

```
ogrinfo /tmp/tilecache/ne_countries/0/0/0.pbf 
INFO: Open of `/tmp/tilecache/ne_countries/0/0/0.pbf'
      using driver `MVT' successful.
1: country (Multi Polygon)
2: country-name (Point)
3: land-border-country (Multi Line String)
4: state
5: diagnostics-tile (Polygon)
6: diagnostics-label (Point)
```

Inspect a single layer:
```
ogrinfo /tmp/tilecache/ne_countries/0/0/0.pbf country-name
INFO: Open of `/tmp/tilecache/ne_countries/0/0/0.pbf'
      using driver `MVT' successful.

Layer name: country-name
Geometry: Point
Feature Count: 298
Extent: (-19743990.154174, -15869550.064455) - (19871181.369241, 14852020.343923)
Layer SRS WKT:
PROJCRS["WGS 84 / Pseudo-Mercator",
[...]
mvt_id: Integer64 (0.0)
abbrev: String (0.0)
name: String (0.0)
OGRFeature(country-name):0
  abbrev (String) = Indo.
  name (String) = Indonesia
  POINT (13051775.4537504 -244598.490512565)
```

Convert MVT geometries and properties to GeoJSON:
```
ogr2ogr -f GeoJSON layer.json /tmp/tilecache/ne_countries/0/0/0.pbf country
```

### Inspect tiles with QGIS

QGIS is using the GDAL driver to read MVT tiles as a regular geometry source.

* Open a new project
* Drag and drop a vector tile (file ending `pbf`) into the map canvas, or open it in the QGIS Browser
* Select layers to add

### Inspect tile services with QGIS

QGIS is also able to include an tile service as QGIS layer. 

Vector tile services can be added via `Layer` -> `Add Layer` -> `Add Vector Tile Layer`. Add a new generic connection with an URL
like `http://localhost:8080/xyz/ne_countries/{z}/{x}/{y}.pbf`. QGIS also supports adding a style URL.

Raster tile services can be added via `Layer` -> `Add Layer` -> `Add XYZ Layer`. Add a connection with an URL like
`http://localhost:8080/xyz/ne_extracts/{z}/{x}/{y}.png`.

Remark: QGIS supports the Web Mercator tile grid only.

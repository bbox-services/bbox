# Tile Server Reference

* `grid[]` [Grid](#grid): Custom grid definitions
* `datasource[]` [NamedDatasource](#nameddatasource)
* `tileset[]` [TileSet](#tileset): Tileset configurations
* `tilestore[]` [TileCacheProvider](#tilecacheprovider)

## Grid

Custom grid definition
* `json` *String*: Grid JSON file path

## TileSet

Tileset configuration
* `name` *String*: Tileset name, visible part of endpoint
* `tms` *String* (optional): Tile matrix set identifier (Default: `WebMercatorQuad`)
* [Source](#source): Tile source
* `cache` *String* (optional): Tile cache name (Default: no cache)
* `cache_format` *String* (optional): Tile format in store. Defaults to `png` for raster and `pbf` for vector tiles
* `cache_limits` [CacheLimit](#cachelimit) (optional): Optional limits of zoom levels which should be cached. Tiles in other zoom levels are served from live data.

### Source

Tile sources

#### wms_proxy

Raster tiles from external WMS
* `source` *String*: Name of `wms_proxy` datasource
* `layers` *String*

#### map_service

Raster tiles from map service
* `project` *String*
* `suffix` *String*
* `layers` *String*
* `params` *String* (optional): Additional WMS params like transparent=true
* `tile_size` *u16* (optional): Width and height of tile. Defaults to grid tile size (usually 256x256)

#### postgis

PostGIS datasource
* `datasource` *String* (optional): Name of `postgis` datasource (Default: first with matching type)
* `extent` [Extent](#extent) (optional)
* `minzoom` *u8* (optional): Minimum zoom level for which tiles are available (Default: 0). If unset, minzoom is deduced from layer and query minzoom limits.
* `maxzoom` *u8* (optional): Maximum zoom level for which tiles are available (Default: 22).

If unset, maxzoom is deduced from layer and query maxzoom limits.
Viewers use data from tiles at maxzoom when displaying the map at higher zoom levels.
* `center` *Option* (optional): Longitude, latitude of map center (in WGS84).

Viewers can use this value to set the default location.
* `start_zoom` *u8* (optional): Start zoom level. Must be between minzoom and maxzoom.
* `attribution` *String* (optional): Acknowledgment of ownership, authorship or copyright.
* `postgis2` *bool* (optional): PostGIS 2 compatible query (without ST_AsMVT)
* `diagnostics` [TileDiagnostics](#tilediagnostics) (optional): Add diagnostics layer
* `layer[]` [VectorLayer](#vectorlayer): Layer definitions

#### mbtiles

Tiles from MBTile archive
* `path` *Path*

#### pmtiles

Tiles from PMTile archive
* `path` *Path*

### CacheLimit

Tile cache limits
* `minzoom` *u8* (optional)
* `maxzoom` *u8* (optional)

#### Extent

* `minx` *f64*
* `miny` *f64*
* `maxx` *f64*
* `maxy` *f64*

#### TileDiagnostics

* `reference_size` *u64* (optional): Maximal tile size (uncompressed)

#### VectorLayer

PostGIS vector layer
* `name` *String*: Layer name.
* `geometry_field` *String* (optional): Name of geometry field.
* `geometry_type` *String* (optional): Type of geometry in PostGIS database

`POINT` | `MULTIPOINT` | `LINESTRING` | `MULTILINESTRING` | `POLYGON` | `MULTIPOLYGON` | `COMPOUNDCURVE` | `CURVEPOLYGON`
* `srid` *i32* (optional): Spatial reference system (PostGIS SRID)
* `no_transform` *bool* (optional): Assume geometry is in grid SRS
* `fid_field` *String* (optional): Name of feature ID field
* `table_name` *String* (optional): Select all fields from table (either table or `query` is required)
* `query[]` [VectorLayerQuery](#vectorlayerquery) (optional): Custom queries
* `minzoom` *u8* (optional): Minimal zoom level for which tiles are available.
* `maxzoom` *u8* (optional): Maximum zoom level for which tiles are available.
* `query_limit` *u32* (optional): Maximal number of features to read for a single tile.
* `tile_size` *u32* (optional): Width and height of the tile (Default: 4096. Grid default size is 256)
* `buffer_size` *u32* (optional): Tile buffer size in pixels (None: no clipping)
* `simplify` *bool* (optional): Simplify geometry (lines and polygons)
* `tolerance` *String* (optional): Simplification tolerance (default to `!pixel_width!/2`)
* `make_valid` *bool* (optional): Fix invalid geometries before clipping (lines and polygons)
* `shift_longitude` *bool* (optional): Apply ST_Shift_Longitude to (transformed) bbox

#### VectorLayerQuery

* `minzoom` *u8* (optional): Minimal zoom level for using this query.
* `maxzoom` *u8* (optional): Maximal zoom level for using this query.
* `simplify` *bool* (optional): Simplify geometry (override layer default setting)
* `tolerance` *String* (optional): Simplification tolerance (override layer default setting)
* `sql` *String* (optional): User defined SQL query.

The following variables are replaced at runtime:
* `!bbox!`: Bounding box of tile
* `!zoom!`: Zoom level of tile request
* `!x!`, `!y!`: x, y of tile request (disables geometry filter)
* `!scale_denominator!`: Map scale of tile request
* `!pixel_width!`: Width of pixel in grid units
* `!<fieldname>!`: Custom field query variable

## TileCacheProvider

* `name` *String*: Name of tile cache
* [TileStore](#tilestore): Tile store

### TileStore

Tile stores

#### files

File system tiles store
* `base_dir` *Path*: Base directory, tileset name will be appended

#### s3

S3 tile store
* `path` *String*

#### mbtiles

MBTile archive
* `path` *Path*

#### pmtiles

PMTile archive
* `path` *Path*

#### nostore

Disable tile cache

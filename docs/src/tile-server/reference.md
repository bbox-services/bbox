# Tile Server Reference

* `grid[]` [Grid](#grid)
* `datasource[]` [NamedDatasource](#nameddatasource)
* `tileset[]` [TileSet](#tileset)
* `tilestore[]` [TileCacheProvider](#tilecacheprovider)

## Grid

Custom grid definition
* `json` *String*: JSON file path

## TileSet

* `name` *String*
* `tms` *String* (optional): List of available tile matrix set identifiers (Default: WebMercatorQuad)
* [Source](#source): Source parameters
* `cache` *String* (optional): Tile cache name (Default: no cache)
* `cache_format` *String* (optional): tile format in store. Defaults to `png` for raster and `pbf` for vector tiles
* `cache_limits` [CacheLimit](#cachelimit) (optional)

### Source


#### wms_proxy

* `source` *String*: name of WmsHttpSourceProviderCfg
* `layers` *String*

#### map_service

* `project` *String*
* `suffix` *String*
* `layers` *String*
* `params` *String* (optional): Additional WMS params like transparent=true
* `tile_size` *u16* (optional): Width and height of tile. Defaults to grid tile size (usually 256x256)

#### postgis

* `datasource` *String* (optional): Name of tileserver.source config (Default: first with matching type)
* `extent` [Extent](#extent) (optional)
* `minzoom` *u8* (optional)
* `maxzoom` *u8* (optional)
* `center` *Option* (optional)
* `start_zoom` *u8* (optional)
* `attribution` *String* (optional)
* `postgis2` *bool* (optional): PostGIS 2 compatible query (without ST_AsMVT)
* `diagnostics` [TileDiagnostics](#tilediagnostics) (optional): Add diagnostics layer
* `layer[]` [VectorLayer](#vectorlayer)

#### mbtiles

* `path` *Path*

#### pmtiles

* `path` *Path*

### CacheLimit

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

* `name` *String*
* `geometry_field` *String* (optional)
* `geometry_type` *String* (optional)
* `srid` *i32* (optional): Spatial reference system (PostGIS SRID)
* `no_transform` *bool* (optional): Assume geometry is in grid SRS
* `fid_field` *String* (optional): Name of feature ID field
* `table_name` *String* (optional)
* `query_limit` *u32* (optional)
* `query[]` [VectorLayerQuery](#vectorlayerquery) (optional)
* `minzoom` *u8* (optional)
* `maxzoom` *u8* (optional)
* `tile_size` *u32* (optional): Width and height of the tile (Default: 4096. Grid default size is 256)
* `simplify` *bool* (optional): Simplify geometry (lines and polygons)
* `tolerance` *String* (optional): Simplification tolerance (default to !pixel_width!/2)
* `buffer_size` *u32* (optional): Tile buffer size in pixels (None: no clipping)
* `make_valid` *bool* (optional): Fix invalid geometries before clipping (lines and polygons)
* `shift_longitude` *bool* (optional): Apply ST_Shift_Longitude to (transformed) bbox

#### VectorLayerQuery

* `minzoom` *u8* (optional)
* `maxzoom` *u8* (optional)
* `simplify` *bool* (optional): Simplify geometry (override layer default setting)
* `tolerance` *String* (optional): Simplification tolerance (override layer default setting)
* `sql` *String* (optional)

## TileCacheProvider

* `name` *String*
* [TileStore](#tilestore)

### TileStore


#### files

* `base_dir` *Path*: Base directory, tileset name will be appended

#### s3

* `path` *String*

#### mbtiles

* `path` *Path*

#### pmtiles

* `path` *Path*

#### nostore


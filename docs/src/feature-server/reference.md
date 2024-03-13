# Feature Server Reference

* `datasource[]` [NamedDatasource](#nameddatasource)
* `collections` [Collections](#collections)
* `collection[]` [ConfiguredCollection](#configuredcollection)

## Collections

Collections with auto-detection
* `directory[]` [DsFiledir](#dsfiledir)
* `postgis[]` [DsPostgis](#dspostgis)

### DsFiledir

* `dir` *String*

## ConfiguredCollection

* `name` *String*
* `title` *String* (optional)
* `description` *String* (optional)
* [CollectionSource](#collectionsource)

### CollectionSource

Collections with configuration

#### postgis

* `datasource` *String* (optional): Name of datasource.postgis config (Default: first with matching type)
* `table_schema` *String* (optional)
* `table_name` *String* (optional)
* `sql` *String* (optional): Custom SQL query
* `fid_field` *String* (optional)
* `geometry_field` *String* (optional)
* `temporal_field` *String* (optional)
* `temporal_end_field` *String* (optional)
* `queryable_fields` *Vec* (optional)

#### gpkg

* `datasource` *String* (optional): Name of datasource.gpkg config (Default: first with matching type)
* `table_name` *String* (optional)
* `sql` *String* (optional): Custom SQL query
* `fid_field` *String* (optional)
* `geometry_field` *String* (optional)

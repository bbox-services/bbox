# Map Server Reference

* `fcgi_client_pool_size` *usize*
* `wait_timeout` *u64* (optional)
* `create_timeout` *u64* (optional)
* `recycle_timeout` *u64* (optional)
* `qgis_backend` [QgisBackend](#qgisbackend) (optional)
* `umn_backend` [UmnBackend](#umnbackend) (optional)
* `mock_backend` [MockBackend](#mockbackend) (optional)
* `search_projects` *bool*
* `default_project` *String* (optional)

## QgisBackend

* `exe_location` *String* (optional)
* `project_basedir` *String*
* `qgs` [QgisBackendSuffix](#qgisbackendsuffix) (optional)
* `qgz` [QgisBackendSuffix](#qgisbackendsuffix) (optional)

### QgisBackendSuffix

* `path` *String*

### QgisBackendSuffix

* `path` *String*

## UmnBackend

* `exe_location` *String* (optional)
* `project_basedir` *String*
* `path` *String*

## MockBackend

* `path` *String*

# Core Reference

* `webserver` [Webserver](#webserver) (optional)
* `metrics` [Metrics](#metrics) (optional)
* `datasource[]` [NamedDatasource](#nameddatasource)
* `auth` [Auth](#auth) (optional)

## Webserver

* `server_addr` *String*
* `tls_cert` *String* (optional)
* `tls_key` *String* (optional)
* `cors` [Cors](#cors) (optional)

### Cors

* `allow_all_origins` *bool*

## Metrics

* `prometheus` [Prometheus](#prometheus) (optional)
* `jaeger` [Jaeger](#jaeger) (optional)

### Prometheus

* `path` *String*

### Jaeger

* `agent_endpoint` *String*

## NamedDatasource

* `name` *String*
* [Datasource](#datasource)

### Datasource


#### postgis

* `url` *String*

#### gpkg

* `path` *Path*

#### WmsFcgi


#### wms_proxy

* `baseurl` *String*
* `format` *String*

#### mbtiles


## Auth

* `oidc` [OidcAuth](#oidcauth) (optional)

### OidcAuth


# Core Reference

* `webserver` [Webserver](#webserver) (optional)
* `metrics` [Metrics](#metrics) (optional)
* `datasource[]` [NamedDatasource](#nameddatasource) (optional)
* `auth` [Auth](#auth) (optional)

## Webserver

* `server_addr` *String*: IP address of interface and port to bind web server (e.g. 0.0.0.0:8080 for all)
* `loglevel` *Loglevel* (optional): Log level (Default: info)
* `tls_cert` *String* (optional)
* `tls_key` *String* (optional)
* `cors` [Cors](#cors) (optional)

### Loglevel


#### Error


#### Warn


#### Info


#### Debug


#### Trace


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


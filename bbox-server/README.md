BBOX server
===========

BBOX spatial services with QWC2 map viewer.


Usage
-----

    cargo run

    x-www-browser http://127.0.0.1:8080/

    x-www-browser http://127.0.0.1:8080//data/usergrid.html?debug=1

Configuration
-------------

Configuraton is read from `bbox.toml` and environment variables.

## Webserver

```toml
[webserver]
# Web server settings
# Environment variable prefix: BBOX_WEBSERVER__
# server_addr = "127.0.0.1:8080"  # Default: 127.0.0.1:8080
# worker_threads = 4  # Default: number of CPU cores

[[fileserver.static]] 
# Static file serving
# Env var example: BBOX_FILESERVER__STATIC='[{dir="data",path="data"}]'
# ./data/* -> http://localhost:8080/data/
# dir = "./data"
# path = "data"

[[fileserver.repo]]
# QGIS plugin repository
# Env var example: BBOX_FILESERVER__REPO='[{dir="plugins",path="qgis"}]'
# ./plugins/*.zip -> http://localhost:8080/qgis/plugins.xml
# dir = "./plugins"
# path = "qgis"

[wmsserver]
# WMS server settings
# Environment variable prefix: BBOX_WMSSERVER__
path = "/wms"                # Base path of WMS endpoints
# num_fcgi_processes = 4     # Default: number of CPU cores
# fcgi_client_pool_size = 1  # FCGI client pool size. Default: 1
search_projects = false      # Scan directories and build inventory

[wmsserver.qgis]
# QGIS Server settings
# Environment variable prefix: BBOX_WMSSERVER__QGIS_BACKEND__
# project_basedir = "."      # Base dir for project files (.qgs, .qgz)

[wmsserver.umn]
# UMN MapServer settings
# Environment variable prefix: BBOX_WMSSERVER__UMN_BACKEND__
# project_basedir = "."      # Base dir for project files (.map)

[wmsserver.mock]
# Enable FCGI mockup backend (for testing)
# Environment variable prefix: BBOX_WMSSERVER__MOCK_BACKEND__

[metrics.prometheus]
# Prometheus metrics endpoint
# Environment variable prefix: BBOX_METRICS__PROMETHEUS__
path = "/metrics"

[metrics.jaeger] 
# Jaeger tracing
# Environment variable prefix: BBOX_METRICS__JAEGER__
agent_endpoint = "localhost:6831"
```


Instrumentation
---------------

### Prometheus

https://prometheus.io/

Run Prometheus:

    docker run --rm -p 127.0.0.1:9090:9090 -v $PWD/instrumentation/prometheus.yml:/etc/prometheus/prometheus.yml:ro prom/prometheus

Test expression browser:

    x-www-browser http://localhost:9090/

Expression example:

    http_requests_duration_bucket


### Jaeger tracing

Run jaeger in background:

    docker run --rm -d -p 6831:6831/udp -p 6832:6832/udp -p 16686:16686 jaegertracing/all-in-one:latest

View spans:

    x-www-browser http://localhost:16686/


### Grafana

https://grafana.com/docs/grafana/

Run Grafana:

    docker run -rm -p 127.0.0.1:3000:3000 grafana/grafana

Open Grafana:

    x-www-browser http://localhost:3000/

- Enter `admin` for username and password
- Add Prometheus datasource with URL http://172.17.0.1:9090/
- Add Jaeger datasource with URL http://172.17.0.1:16686/

Average request duration:

    rate(http_requests_duration_sum[5m])/rate(http_requests_duration_count[5m])

Request duration 90th percentile
        
    histogram_quantile(0.9, rate(http_requests_duration_bucket[5m]))

https://www.robustperception.io/how-does-a-prometheus-histogram-work

WMS Endpoint:

    http_requests_duration_sum{endpoint="/qgis/{project:.+}"}

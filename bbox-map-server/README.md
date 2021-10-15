BBOX map server
===============

Asynchronous map server with FCGI backend.

Features:
- [ ] OGC WMS 1.3 Server
- [ ] OGC Map API Server
- [ ] Instrumentation: Prometheus and Jaeger tracing
- [X] Map rendering backends: QGIS Server + UNN Mapserver
- [ ] Intelligent process dispatching (slow query detection)

Usage
-----

    cd ..
    cargo run

Request examples:

    curl 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&REQUEST=GetCapabilities'

    curl -s -v -o /tmp/map.png 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png;%20mode%3D8bit'

    curl -s -v -o /tmp/legend.png 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetLegendGraphic&LAYER=country&FORMAT=image/png&STYLE=default&TRANSPARENT=true'


Instrumenation
-------------

### Prometheus

https://prometheus.io/

Run Prometheus:

    docker run --rm -p 127.0.0.1:9090:9090 -v $PWD/instrumentation/prometheus.yml:/etc/prometheus/prometheus.yml:ro prom/prometheus

Test expression browser:

    x-www-browser http://localhost:9090/

Expression example:

    wmsapi_http_requests_duration_seconds_bucket


### Jaeger tracing

Run jaeger in background:

    docker run --rm -d -p 6831:6831/udp -p 6832:6832/udp -p 16686:16686 -p 14268:14268 jaegertracing/all-in-one:latest

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

    rate(wmsapi_http_requests_duration_seconds_sum[5m])/rate(wmsapi_http_requests_duration_seconds_count[5m])

Request duration 90th percentile
        
    histogram_quantile(0.9, rate(wmsapi_http_requests_duration_seconds_bucket[5m]))

https://www.robustperception.io/how-does-a-prometheus-histogram-work

WMS Endpoint:

    wmsapi_http_requests_duration_seconds_sum{endpoint="/wms/qgs/{project:.+}"}


Development
-----------

Documentation of used libriaries:
* Actix: https://actix.rs/
* Async Process: https://docs.rs/async-process/
* QGIS Server plugins: https://docs.qgis.org/3.10/en/docs/user_manual/working_with_ogc/server/plugins.html

Fast CGI:
* Fast CGI: https://fastcgi-archives.github.io/FastCGI_Specification.html
* CGI: https://tools.ietf.org/html/rfc3875

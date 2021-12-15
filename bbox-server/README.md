BBOX server
===========

BBOX spatial services with QWC2 map viewer.


Usage
-----

    cargo run

    x-www-browser http://127.0.0.1:8080/


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

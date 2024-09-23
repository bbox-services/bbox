---
weight: 12
---

# Instrumentation

## Configuration

### Prometheus metrics

```toml
[metrics.prometheus]
# Prometheus metrics endpoint
# Environment variable prefix: BBOX_METRICS__PROMETHEUS__
path = "/metrics"
```

### Jaeger tracing

```toml
[metrics.jaeger]
# Environment variable prefix: BBOX_METRICS__JAEGER__
agent_endpoint = "localhost:6831"
```

## Applications

### Prometheus

<https://prometheus.io/>

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

<https://grafana.com/docs/grafana/>

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

<https://www.robustperception.io/how-does-a-prometheus-histogram-work>

WMS Endpoint:

    http_requests_duration_sum{endpoint="/qgis/{project:.+}"}

Docker Compose Environment
==========================

Setup
-----

    cp template.env .env
    echo "PG_PASS=$(pwgen -s 40 1)" >> .env
    echo "AUTHENTIK_SECRET_KEY=$(pwgen -s 50 1)" >> .env

Usage
-----

    docker-compose -f docker-compose-bbox.yml -f docker-compose-postgresql.yml -f docker-compose-nginx.yml -f docker-compose-auth.yml -f docker-compose-windmill.yml -f docker-compose-metrics.yml up -d

Or with Make installed:

    make up

### BBOX

    x-www-browser http://127.0.0.1:8080/

### Authentik

    x-www-browser http://127.0.0.1:9000/

Login: akadmin
Password: akadmin

### Windmill

    x-www-browser http://127.0.0.1:8000/

Email: admin@windmill.dev
Password: changeme


Instrumenation
-------------

### Grafana

https://grafana.com/docs/grafana/

Open Grafana:

    x-www-browser http://127.0.0.1:8080/grafana/

- Enter `admin` for username and password


BBOX WMS metrics examples:

Average request duration

    rate(http_requests_duration_sum[5m])/rate(http_requests_duration_count[5m])

Request duration 90th percentile
        
    histogram_quantile(0.9, rate(http_requests_duration_bucket[5m]))

https://www.robustperception.io/how-does-a-prometheus-histogram-work

WMS Endpoint:

    http_requests_duration_sum{endpoint="/wms/qgs/{project:.+}"}


### Jaeger tracing

View spans:

    x-www-browser http://127.0.0.1:16686/

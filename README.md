# BBOX services

[![CI build](https://github.com/sourcepole/bbox/workflows/CI/badge.svg)](https://github.com/sourcepole/bbox/actions)
[![Docker](https://img.shields.io/docker/v/sourcepole/bbox-server-qgis?label=Docker%20image&sort=semver)](https://hub.docker.com/r/sourcepole/bbox-server-qgis)

Composable spatial services.

Features:
* OGC WMS Server (backends: QGIS Server + UNN Mapserver)
* Built-in map viewer
* Static file server
* Instrumentation: Prometheus and Jaeger tracing
* Healths endpoints for Kubernetes hosting

Components:
* [BBOX map server](bbox-map-server/)
* [BBOX map viewer](bbox-map-viewer/)
* [BBOX OGC API Features service](bbox-feature-server/)
* [BBOX asset server](bbox-asset-server/)


## Build and run

    cd bbox-server
    cargo install --path .
    ~/.cargo/bin/bbox-server


## Docker

    docker build -f ./Dockerfile-qgis-server -t bbox .

Run with environment variables:

    docker run -p 8080:8080 -e BBOX_WEBSERVER__SERVER_ADDR=0.0.0.0:8080 bbox

Run with configuration file:

    docker run -p 8080:8080 -v bbox.toml:/var/www/bbox.toml:ro bbox

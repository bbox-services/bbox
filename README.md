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

    docker run -p 8080:8080 sourcepole/bbox-server-qgis

Serve tiles from file:

    docker run -p 8080:8080 -v $PWD/assets:/assets:ro sourcepole/bbox-server-qgis bbox-server serve /assets/liechtenstein.mbtiles

Run with configuration file:

    docker run -p 8080:8080 -v $PWD/bbox.toml:/var/www/bbox.toml:ro -v $PWD/assets:/assets:ro sourcepole/bbox-server-qgis

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

# BBOX services

Composable spatial services.

[![Documentation](https://img.shields.io/badge/docs-Book-informational)](https://www.bbox.earth/)
[![CI build](https://github.com/sourcepole/bbox/workflows/CI/badge.svg)](https://github.com/sourcepole/bbox/actions)
[![Docker](https://img.shields.io/docker/v/sourcepole/bbox-server-qgis?label=Docker%20image&sort=semver)](https://hub.docker.com/r/sourcepole/bbox-server-qgis)

```
 ___ ___  _____  __
| _ ) _ )/ _ \ \/ /
| _ \ _ \ (_) >  < 
|___/___/\___/_/\_\
```

Components:
* [BBOX Feature server](bbox-feature-server): OGC API Features service
* [BBOX Map server](bbox-map-server): OGC API Map service
* [BBOX Tile server](bbox-tile-server): OGC API Tile service
* [BBOX Asset server](bbox-asset-server): Serving static and templated files
* [BBOX Processes server](bbox-processes-server): OGC API Processes service
* [BBOX Routing server](bbox-routing-server): OGC API Routing service (experimental)

Features:
* Built-in high performance HTTP server
* QWC2 Map viewer
* Instrumentation: Prometheus and Jaeger tracing
* Healths endpoints for Docker and Kubernetes hosting


See [Documentation](https://www.bbox.earth/) for detailed information.

## Installation

See [Documentation](https://www.bbox.earth/installation.html) for instructions.

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

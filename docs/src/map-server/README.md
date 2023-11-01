# BBOX map server

Asynchronous map server with FCGI backend.

Features:
- [x] OGC WMS 1.3 Server
- [ ] OGC Map API Server
- [x] FCGI backends:
  - [X] QGIS Server
  - [X] UNN Mapserver
- [ ] Instrumentation data for WMS backends
- [ ] Intelligent process dispatching (slow query detection)


## Usage

Run feature server with `bbox.toml` configuration:

    bbox-map-server serve

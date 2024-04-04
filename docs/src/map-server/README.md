# BBOX map server

Asynchronous map server with FCGI backend.

Features:
- [x] OGC WMS 1.3 Server
- [ ] OGC API – Maps (Draft)
- [x] FCGI backends:
  - [x] QGIS Server
  - [x] UNN Mapserver
- [x] Instrumentation data for WMS backends
- [x] FCGI dispatcher optimized for WMS requests


## Usage

Run feature server with `bbox.toml` configuration:

    bbox-map-server serve

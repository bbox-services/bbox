---
weight: 11
next: docs/routing-server/configuration
---

# BBOX Routing Service (EXPERIMENTAL)

Routing services with Contraction Hierarchy.

Features:
- [ ] OGC API - Routes - Part 1: Core
- [x] Multiple search APIs
  - [x] OGC API route requests
  - [x] Basic from/to requests
  - [x] Valhalla API compatible requests
- [x] Builtin storage backends: PostGIS, GeoPackage
- [ ] Extract routing graphs from OSM planet files


## Usage

Run tile server with `bbox.toml` configuration:

    bbox-routing-server serve

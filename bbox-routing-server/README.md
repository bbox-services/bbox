BBOX Routing Service
====================

Routing services with Contraction Hierarchy.

Features:
- [ ] OGC API - Routes - Part 1: Core
- [ ] OpenAPI endpoint
- [ ] GeoZero backend (PostGIS, GeoPackage, ...)
- [ ] Extract routing graphs from OSM planet files


Usage
-----

Request examples:

    curl -s -X 'POST' \
      'http://localhost:8080/routes?mode=sync' \
      -H 'accept: application/geo+json' \
      -H 'Content-Type: application/json' \
      -d '{
      "name": "Route from A to B",
      "waypoints": {
        "type": "MultiPoint",
        "coordinates": [
          [9.35213353, 47.0935012],
          [9.3422712, 47.1011887]
        ]
      },
      "preference": "fastest",
      "dataset": "OSM"
    }'

Basic from/to request:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=9.35213353,47.0935012&to_pos=9.3422712,47.1011887'

Zurich - Munich:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=8.53636,47.37726&to_pos=11.56096,48.14019'


Valhalla endpoint (e.g. for Valhalla QGIS Plugin):

Base URL: http://localhost:8080/routes/valhalla

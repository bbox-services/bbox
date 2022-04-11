BBOX Routing Service
====================

Routing services with Contraction Hierarchy .

Features:
- [ ] OGC API - Routes - Part 1: Core
- [ ] OpenAPI endpoint
- [ ] GeoZero backend (PostGIS, GeoPackage, ...)
- [ ] Extract routing graphs from OSM planet files


Usage
-----

Request examples:

    curl -X 'POST' \
      'http://localhost:8080/routes?mode=sync' \
      -H 'accept: application/geo+json' \
      -H 'Content-Type: application/json' \
      -d '{
      "name": "Route from A to B",
      "waypoints": {
        "type": "MultiPoint",
        "coordinates": [
          [36.1234515, 32.6453783],
          [36.1247213, 32.7106286]
        ]
      },
      "preference": "fastest",
      "dataset": "OSM"
    }'

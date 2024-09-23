---
weight: 4
---

# BBOX API Endpoints

Services are available via the following HTTP endpoints:

|                  URL                  |         Description         |
|---------------------------------------|-----------------------------|
| `/tiles`                              | List of available tilesets  |
| `/tiles/{tileset}`                    | Tileset metadata            |
| `/map/tiles/{grid}/{z}/{x}/{y}`       | Map tiles endpoint          |
| `/xyz/{tileset}/{z}/{x}/{y}.{format}` | XYZ tile endpoint           |
| `/xyz/{tileset}.json`                 | Tilejson endpoint           |
| `/xyz/{tileset}.style.json`           | Generic Style JSON endpoint |
| `/xyz/{tileset}/metadata.json`        | MBTiles metadata JSON       |

## Request examples

Tile requests:

    curl -o /tmp/tile.png http://localhost:8080/xyz/ne_extracts/2/2/2.png

    curl -o /tmp/tile.png http://localhost:8080/xyz/ne_umn/2/2/2.png

    curl -o /tmp/tile.jpg http://localhost:8080/xyz/gebco/0/0/0.jpeg

    curl -o /tmp/tile.mvt http://localhost:8080/xyz/mbtiles_mvt_fl/14/8621/5759.mvt

    curl -o /tmp/tilegz.mvt -H 'Content-Encoding: gzip' http://localhost:8080/xyz/mbtiles_mvt_fl/14/8621/5759.mvt

    curl -o /tmp/tile.png -H 'Accept: image/png; mode=8bit' http://localhost:8080/map/tiles/WebMercatorQuad/2/2/2

    curl -o /tmp/tile.mvt http://localhost:8080/xyz/liechtenstein/14/8621/5759.mvt

XYZ URL (Leaflet, QGIS, etc.):

    http://localhost:8080/xyz/ne_extracts/{z}/{x}/{y}.png

Tilejson requests:

    curl -s http://localhost:8080/xyz/mbtiles_mvt_fl.json | jq .

Style JSON requests:

    curl -s http://localhost:8080/xyz/mbtiles_mvt_fl.style.json | jq .

    curl -s http://localhost:8080/xyz/ne_extracts.style.json | jq .

Map viewer examples:

    x-www-browser http://127.0.0.1:8080/assets/usergrid.html?debug=1

Map viewer template examples:

    x-www-browser http://localhost:8080/html/maplibre/mbtiles_mvt_fl?style=/assets/mbtiles_mvt_fl-style.json

With PostGIS Service:

    docker run -p 127.0.0.1:5439:5432 -d --name mvtbenchdb --rm sourcepole/mvtbenchdb

    curl -s http://localhost:8080/xyz/ne_countries.style.json | jq .
    x-www-browser http://localhost:8080/html/maplibre/ne_countries

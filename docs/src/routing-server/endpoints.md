# BBOX API Endpoints

Services are available via the following HTTP endpoints:

|        URL         |         Description          |
|--------------------|------------------------------|
| `/routes`          | OGC API endpoint             |
| `/routes/basic`    | Basic from/to API endpoint   |
| `/routes/valhalla` | Valhalla compatible endpoint |


## Request examples

### OGC API

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

### Basic from/to request:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=9.35213353,47.0935012&to_pos=9.3422712,47.1011887'

Zurich - Munich:

    curl -s 'http://localhost:8080/routes/basic?profile=railway&from_pos=8.53636,47.37726&to_pos=11.56096,48.14019'


### Valhalla endpoint

Base URL example for Valhalla QGIS Plugin: http://localhost:8080/routes/valhalla

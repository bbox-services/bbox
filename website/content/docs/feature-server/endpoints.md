# BBOX API Endpoints

Services are available via the HTTP `GET` endpoints:

|               URL                |     Description     |
|----------------------------------|---------------------|
| `/collections`                   | List of collections |
| `/collections/{name}/items`      | Collection items    |
| `/collections/{name}/items/{id}` | Single item         |


## Request examples

Inspect collections:

    x-www-browser http://127.0.0.1:8080/collections

Feature requests:

    curl -s http://127.0.0.1:8080/collections/populated_places/items | jq .

    curl -s http://127.0.0.1:8080/collections/populated_places_names/items/2 | jq .

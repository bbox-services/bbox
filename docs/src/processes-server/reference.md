# Processes Server Reference

* `dagster_backend` [DagsterBackend](#dagsterbackend) (optional)

## DagsterBackend

Dagster backend configuration
* `graphql_url` *String*: GraphQL URL (e.g. `http://localhost:3000/graphql`)
* `repository_name` *String*: Dagster repository (e.g. `fpds2_processing_repository`)
* `repository_location_name` *String*: Dagster repository location (e.g. `fpds2_processing.repos`)
* `request_timeout` *u64* (optional): Backend request timeout (ms) (Default: 10s)

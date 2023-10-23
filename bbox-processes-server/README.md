# BBOX Processes Service

The OGC API - Processes standard specifies an interface for executing computational tasks.

Overview: https://ogcapi.ogc.org/processes/

Features:
- [ ] OGC API - Processes - Part 1: Core
- [x] Support synchronous and asynchronous process execution
- [x] OpenAPI endpoint
- [x] Multiple backend engines
  - [x] [Dagster](https://dagster.io/)
  - [ ] [Windmill](https://www.windmill.dev/)


## Configuration

Dagster Backend:
```toml
[processes.dagster_backend]
graphql_url = "http://localhost:3000/graphql"
repository_name = "the_repository"
repository_location_name = "the.repos"
```

## Usage

### Request examples

List processes:

    curl 'http://localhost:8080/processes'

Execute process:

    curl --header "Content-Type: application/json" \
         --request POST \
         --data '{"inputs": {"ops": {"pos_info_query": {"inputs": {"pos_x": 2607545, "pos_y": 1171421}}}}}' \
      http://localhost:8080/processes/pos_info/execution

Execute process asynchronous:

    curl --header "Content-Type: application/json" \
         --header "Prefer: respond-async" \
         --request POST \
         --data '{"inputs": {"ops": {"export_fpds2": {"inputs": {"fixpunkte": ["12575280", "12575100"], "in_bearbeitung": false }}}}}' \
      http://localhost:8080/processes/export_fpds2_to_csv/execution

    JOBID=386f6c55-d718-4160-b4df-afc5ad5c7a73

Get job status:

    curl http://localhost:8080/jobs/$JOBID

Return result of a job:

    curl http://localhost:8080/jobs/$JOBID/results

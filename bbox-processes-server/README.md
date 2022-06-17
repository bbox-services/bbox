BBOX Processes Service
======================

The OGC API - Processes standard specifies an interface for executing computational tasks.

Overview: https://ogcapi.ogc.org/processes/

Features:
- [ ] OGC API - Processes - Part 1: Core
- [x] Support synchronous and asynchronous process execution
- [x] OpenAPI endpoint
- [x] [dagster](https://dagster.io/) backend


Usage
-----

### Request examples

List processes:

    curl 'http://localhost:8080/processes'

Execute process:

    curl -v --header "Content-Type: application/json" \
         --request POST \
         --data '{"inputs": {"ops": {"get_gemeinde_json": {"inputs": {"fixpunkt_X": 2607545, "fixpunkt_Y": 1171421}}}}}' \
      http://localhost:8080/processes/get_gemeinde/execution

Execute process asynchronous:

    curl -v --header "Content-Type: application/json" \
         --header "Prefer: respond-async" \
         --request POST \
         --data '{"inputs": {"ops": {"export_fpds2": {"inputs": {"csv_output_path": "/data", "db_table": "fpds2.fixpunkt_work"}}}}}' \
      http://localhost:8080/processes/export_fpds2_to_csv/execution

    JOBID=386f6c55-d718-4160-b4df-afc5ad5c7a73

Get job status:

    curl http://localhost:8080/jobs/$JOBID

Return result of a job:

    curl http://localhost:8080/jobs/$JOBID/results

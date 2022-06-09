BBOX Processes Service
======================

The OGC API - Processes standard specifies an interface for executing computational tasks.

Overview: https://ogcapi.ogc.org/processes/

Features:
- [ ] OGC API - Processes - Part 1: Core
- [ ] OpenAPI endpoint
- [ ] [dagster](https://dagster.io/) backend


Usage
-----

### Request examples

List processes:

    curl 'http://localhost:8080/processes'

Execute process (asynchronous):

    curl -v --header "Content-Type: application/json" \
         --request POST \
         --data '{"inputs": {"ops": {"get_gemeinde_json": {"inputs": {"fixpunkt_X": 2607545, "fixpunkt_Y": 1171421}}}}}' \
      http://localhost:8080/processes/get_gemeinde/execution

Get job status:

    curl 'http://localhost:8080/jobs/4a979b42-5831-4368-9913-685293a22ebc'

Return result of a job:

    curl 'http://localhost:8080/jobs/4a979b42-5831-4368-9913-685293a22ebc/results'

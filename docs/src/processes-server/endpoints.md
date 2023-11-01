# BBOX API Endpoints

Services are available via the following HTTP endpoints:

|              URL              |         Description         |
|-------------------------------|-----------------------------|
| `/processes`                  | List of available processes |
| `/processes/{name}/execution` | Execute processes           |
| `/processes/jobs/{jobid}`     | Job status                  |
| `/processes/{jobid}/results`  | Job results                 |


## Request examples

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

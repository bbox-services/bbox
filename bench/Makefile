sw = asyncfcgi
result_prefix = ./results/results_
wkr_cmd = docker run --rm --user=$$UID --net=host -e SW="$(sw)" -e CSV_NAME=$(csv) -e CONNECTIONS=[CONN] -v $$PWD:/bench -v $$PWD/paths.txt:/paths.txt:ro williamyeh/wrk -H 'Accept-Encoding: gzip' -H 'Connection: keep-alive'
duration = 10

# Run server with
# WMS_BACKEND="Mock FCGI WMS" RUST_LOG=info cargo run --release

quick_bench:
	make bench_http duration=5 csv=http_quick.csv

bench:
	make bench_http csv=http.csv

bench_http:
	@# From first entry only title is displayed on plot
	$(wkr_cmd:[CONN]=1) --latency -d 1 -c 1 --timeout 8 -t 1 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=1) --latency -d $(duration) -c 1 --timeout 8 -t 1 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=4) --latency -d $(duration) -c 4 --timeout 8 -t 4 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=32) --latency -d $(duration) -c 32 --timeout 8 -t 4 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=64) --latency -d $(duration) -c 64 --timeout 8 -t 4 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=128) --latency -d $(duration) -c 128 --timeout 8 -t 4 -s /bench/httpbench.lua http://127.0.0.1:8080
	$(wkr_cmd:[CONN]=256) --latency -d $(duration) -c 256 --timeout 8 -t 4 -s /bench/httpbench.lua http://127.0.0.1:8080
	@echo >>$(result_prefix)${csv}
	@echo >>$(result_prefix)${csv}
	@echo Statistics written to $(result_prefix)${csv}

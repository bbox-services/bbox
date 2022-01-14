# See mock-fcgi-wms/README.md for running server

result_prefix = ./results/results_
path_file = paths.txt
wrk_cmd = docker run --rm --user=$$UID --net=host -e SW="$(sw)" -e CSV_NAME=$(csv) -e CONNECTIONS=[CONN] \
         -v $$PWD:/bench -v $$PWD/$(path_file):/paths.txt:ro  \
         williamyeh/wrk -H 'Accept-Encoding: gzip' -H 'Connection: keep-alive'
duration = 10s

quick_bench_apache:
	make bench_http duration=5s csv=http_quick.csv sw=apache-baseline path_file=paths_baseline.txt host=http://127.0.0.1
	make bench_http duration=5s csv=http_quick.csv sw=apache path_file=paths.txt host=http://127.0.0.1

quick_bench:
	make bench_http duration=5s csv=http_quick.csv sw=bbox-baseline path_file=paths_baseline.txt host=http://127.0.0.1:8080
	sleep 10
	make bench_http duration=5s csv=http_quick.csv sw=bbox path_file=paths.txt host=http://127.0.0.1:8080

bench_apache:
	make bench_http csv=http.csv sw=apache-baseline path_file=paths_baseline.txt host=http://127.0.0.1
	make bench_http csv=http.csv sw=apache path_file=paths.txt host=http://127.0.0.1

bench:
	make bench_http csv=http.csv sw=bbox-baseline path_file=paths_baseline.txt host=http://127.0.0.1:8080
	sleep 20
	make bench_http csv=http.csv sw=bbox path_file=paths.txt host=http://127.0.0.1:8080

bench_http:
	@# From first entry only title is displayed on plot
	$(wrk_cmd:[CONN]=1) --latency -d 1s -c 1 -t 1 -s /bench/httpbench.lua $(host)
	sleep 1
	$(wrk_cmd:[CONN]=1) --latency -d $(duration) -c 1 -t 1 -s /bench/httpbench.lua $(host)
	sleep 1
	$(wrk_cmd:[CONN]=4) --latency -d $(duration) -c 4 -t 4 -s /bench/httpbench.lua $(host)
	sleep 1
	$(wrk_cmd:[CONN]=32) --latency -d $(duration) -c 32 -t 4 -s /bench/httpbench.lua $(host)
	sleep $(duration)
	$(wrk_cmd:[CONN]=64) --latency -d $(duration) -c 64 -t 4 -s /bench/httpbench.lua $(host)
	sleep $(duration)
	$(wrk_cmd:[CONN]=128) --latency -d $(duration) -c 128 -t 4 -s /bench/httpbench.lua $(host)
	sleep $(duration)
	sleep $(duration)
	$(wrk_cmd:[CONN]=256) --latency -d $(duration) -c 256 -t 4 -s /bench/httpbench.lua $(host)
	@echo >>$(result_prefix)${csv}
	@echo >>$(result_prefix)${csv}
	@echo Statistics written to $(result_prefix)${csv}

trace:
	make trace_bench duration=5s csv=http_quick.csv sw=bbox-baseline path_file=paths_baseline.txt host=http://127.0.0.1:8080
	sleep 3
	make trace_bench duration=5s csv=http_quick.csv sw=apache-baseline path_file=paths_baseline.txt host=http://127.0.0.1

trace_bench:
	$(wrk_cmd:[CONN]=32) --latency -d $(duration) -c 32 -t 4 -s /bench/httpbench.lua $(host)
	@echo >>$(result_prefix)${csv}
	@echo >>$(result_prefix)${csv}

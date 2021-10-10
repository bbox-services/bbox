BBOX OGC API Features Service
=============================

Asynchronous OGC API Features server implementation.

Features:
-[ ] OGC API - Features - Part 1: Core 1.0
-[ ] OGC API - Features - Part 2: Coordinate Reference Systems by Reference 1.0
-[ ] GeoZero backend (PostGIS, GeoPackage, ...)
-[ ] Instrumentation: Prometheus and Jaeger tracing


## Usage

Create .env file:
```
SERVER_ADDR=127.0.0.1:8080
PG.USER=t_rex
PG.PASSWORD=t_rex
PG.HOST=127.0.0.1
PG.PORT=5439
PG.DBNAME=t_rex_tests
PG.POOL.MAX_SIZE=16
```

Start test DB:

    docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb

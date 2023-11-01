```
 ___ ___  _____  __
| _ ) _ )/ _ \ \/ /
| _ \ _ \ (_) >  < 
|___/___/\___/_/\_\
```

Composable spatial services.

Components:
* [BBOX Feature server](feature-server/): OGC API Features service
* [BBOX Map server](map-server/): OGC API Map service
* [BBOX Tile server](tile-server/): OGC API Tile service
* [BBOX Asset server](asset-server/): Serving static and templated files
* [BBOX Processes server](processes-server/): OGC API Processes service
* [BBOX Routing server](routing-server/): OGC API Routing service (experimental)

Features:
* Built-in high performance HTTP server
* QWC2 Map viewer
* Instrumentation: Prometheus and Jaeger tracing
* Healths endpoints for Docker and Kubernetes hosting

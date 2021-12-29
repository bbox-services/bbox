BBOX map server
===============

Asynchronous map server with FCGI backend.

Features:
- [x] OGC WMS 1.3 Server
- [ ] OGC Map API Server
- [X] Map rendering backends: QGIS Server + UNN Mapserver
- [ ] Instrumentation data for WMS backends
- [ ] Intelligent process dispatching (slow query detection)

Usage
-----

    cd ..
    cargo run

Configuration:
* `BBOX_WMSSERVER__NUM_FCGI_PROCESSES`: Number of FCGI processes (default: number of CPU cores)


Request examples:

    curl 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&REQUEST=GetCapabilities'

    curl -s -o /tmp/map.png 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png;%20mode%3D8bit'

    curl -s -o /tmp/legend.png 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetLegendGraphic&LAYER=country&FORMAT=image/png&STYLE=default&TRANSPARENT=true'

    curl 'http://127.0.0.1:8080/wms/qgs/helloworld?SERVICE=WMS&REQUEST=GetProjectSettings'

    curl -o /tmp/print.pdf 'http://127.0.0.1:8080/wms/qgs/helloworld' -X POST \
         -d 'SERVICE=WMS&VERSION=1.3.0&REQUEST=GetPrint&FORMAT=pdf' \
         -d 'TEMPLATE=Composer 1&DPI=300&CRS=EPSG:4326' \
         -d 'map0:LAYERS=Country,Hello&map0:extent=-92.8913,-185.227,121.09,191.872'


Development
-----------

Documentation of main libriaries:
* Actix: https://actix.rs/
* Async Process: https://docs.rs/async-process/
* QGIS Server plugins: https://docs.qgis.org/3.10/en/docs/user_manual/working_with_ogc/server/plugins.html

Fast CGI:
* Fast CGI: https://fastcgi-archives.github.io/FastCGI_Specification.html
* CGI: https://tools.ietf.org/html/rfc3875

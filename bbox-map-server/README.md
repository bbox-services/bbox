# BBOX map server

Asynchronous map server with FCGI backend.

Features:
- [x] OGC WMS 1.3 Server
- [ ] OGC Map API Server
- [x] FCGI backends:
  - [X] QGIS Server
  - [X] UNN Mapserver
- [ ] Instrumentation data for WMS backends
- [ ] Intelligent process dispatching (slow query detection)


## Configuration

See [Documentation](https://www.bbox.earth/docs/map-server/configuration/) for examples.

## Usage

    cd ..
    cargo run

Configuration:
* `BBOX_MAPSERVER__NUM_FCGI_PROCESSES`: Number of FCGI processes (default: number of CPU cores)


WMS request examples:

    curl -s 'http://127.0.0.1:8080/qgis/ne?SERVICE=WMS&REQUEST=GetCapabilities'

    curl -o /tmp/map.png 'http://127.0.0.1:8080/qgis/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png;%20mode%3D8bit'

    curl -o /tmp/legend.png 'http://127.0.0.1:8080/qgis/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetLegendGraphic&LAYER=country&FORMAT=image/png&STYLE=default&TRANSPARENT=true'

    curl -s 'http://127.0.0.1:8080/qgis/helloworld?SERVICE=WMS&REQUEST=GetProjectSettings'

    curl -o /tmp/print.pdf 'http://127.0.0.1:8080/qgis/helloworld' -X POST \
         -d 'SERVICE=WMS&VERSION=1.3.0&REQUEST=GetPrint&FORMAT=pdf' \
         -d 'TEMPLATE=Composer 1&DPI=300&CRS=EPSG:4326' \
         -d 'map0:LAYERS=Country,Hello&map0:extent=-92.8913,-185.227,121.09,191.872'

UMN Mapserver:

    curl -s 'http://127.0.0.1:8080/wms/map/ne?SERVICE=WMS&REQUEST=GetCapabilities'

    curl -o /tmp/map.png 'http://127.0.0.1:8080/wms/map/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=40.83354209954528358,0.542981257600549938,49.84069885574058389,15.5221558872974672&CRS=epsg:4326&WIDTH=1372&HEIGHT=825&LAYERS=country&STYLES=&FORMAT=image%2Fpng%3B%20mode%3D8bit'


WFS request examples:

    curl -s 'http://127.0.0.1:8080/qgis/ne?SERVICE=WFS&REQUEST=GetCapabilities'

    curl -s 'http://127.0.0.1:8080/qgis/ne?SERVICE=WFS&REQUEST=GetFeature&VERSION=1.1.0&TYPENAME=country&SRSNAME=EPSG:3857&BBOX=1059483.34824404888786376,5959680.16110791172832251,1061700.73825845750980079,5962445.67000228632241488,EPSG:3857'

    curl -s --data @wfsadd.xml 'http://127.0.0.1:8080/qgis/ne?SERVICE=WFS&REQUEST=Transaction'

## Development

Documentation of main libriaries:
* Actix: https://actix.rs/
* Async Process: https://docs.rs/async-process/
* QGIS Server plugins: https://docs.qgis.org/3.28/en/docs/server_manual/plugins.html

Fast CGI:
* Fast CGI: https://fastcgi-archives.github.io/FastCGI_Specification.html
* CGI: https://tools.ietf.org/html/rfc3875

Asynchronous FCGI Server
========================

Fast CGI
--------

* Fast CGI: https://fastcgi-archives.github.io/FastCGI_Specification.html
* CGI: https://tools.ietf.org/html/rfc3875

Usage
-----

    cargo run

    x-www-browser http://127.0.0.1:8080/

Manual calls:

    curl 'http://127.0.0.1:8080/wms/qgs/ne?&SERVICE=WMS&REQUEST=GetCapabilities'

    curl -s -v -o /tmp/map.png 'http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-20037508.34278924391,-5966981.031407224014,19750246.20310878009,17477263.06060761213&CRS=EPSG:900913&WIDTH=1399&HEIGHT=824&LAYERS=country&STYLES=&FORMAT=image/png;%20mode%3D8bit'

    curl -s -v -o /tmp/legend.png 'http://127.0.0.1:8080/wms/qgs/ne?&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetLegendGraphic&LAYER=country&FORMAT=image/png&STYLE=default&TRANSPARENT=true'

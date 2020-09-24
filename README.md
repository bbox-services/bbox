Asynchronous FCGI Server
========================

Fast CGI
--------

* Fast CGI: https://fastcgi-archives.github.io/FastCGI_Specification.html
* CGI: https://tools.ietf.org/html/rfc3875

Usage
-----

Manual calls:

    curl 'http://127.0.0.1:6767/fcgi/?map=test/helloworld.qgs&SERVICE=WMS&REQUEST=GetCapabilities'

    curl -s -v -o /tmp/map.png 'http://127.0.0.1:6767/fcgi/?map=test/helloworld.qgs&ERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode%3D8bit&DPI=96&TRANSPARENT=TRUE'

    curl -s -v -o /tmp/legend.png 'http://127.0.0.1:6767/fcgi/?map=test/helloworld.qgs&SERVICE=WMS&VERSION=1.3.0&REQUEST=GetLegendGraphic&LAYER=Country&FORMAT=image/png&STYLE=default&SLD_VERSION=1.1.0&TRANSPARENT=true'

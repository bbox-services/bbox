FCGI WMS test server
====================


Standalone usage
----------------

    cargo build

    spawn-fcgi ../../target/debug/mock-fcgi-wms -n -s /tmp/mock-fcgi
    cgi-fcgi -bind -connect /tmp/mock-fcgi

    spawn-fcgi ../../target/debug/mock-fcgi-wms -n -p 8099
    QUERY_STRING='' cgi-fcgi -bind -connect 127.0.0.1:8099


Use with BBOX server
--------------------

    cargo build --release

    cd ../..
    docker-compose up -d

Run server with bbox-map-server/bench/bbox-bench.toml:

    RUST_LOG=info cargo run --release

Test request (50ms):

    curl 'http://localhost:8080/wms/mock/helloworld?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode=8bit&DPI=96&TRANSPARENT=TRUE'

Slow request (1s):

    curl 'http://localhost:8080/wms/mock/slow?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode=8bit&DPI=96&TRANSPARENT=TRUE'

Crash request:

    curl 'http://localhost:8080/wms/mock/crash?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode=8bit&DPI=96&TRANSPARENT=TRUE'

Jaeger tracing:

    x-www-browser http://localhost:16686/

Prometheus metrics:

    x-www-browser http://localhost:9090/

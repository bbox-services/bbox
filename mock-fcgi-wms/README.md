FCGI WMS test server
====================


Standalon usage
---------------

    cargo build

    spawn-fcgi ../target/debug/mock-fcgi-wms -n -s /tmp/afcgi
    cgi-fcgi -bind -connect /tmp/afcgi

    spawn-fcgi ../target/debug/mock-fcgi-wms -n -p 8099
    QUERY_STRING='' cgi-fcgi -bind -connect 127.0.0.1:8099


Jaeger tracing
--------------

    docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest

    cargo run

    firefox http://localhost:16686/

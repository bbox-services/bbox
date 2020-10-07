FCGI WMS test server
====================


Standalon usage
---------------

    cargo build

    spawn-fcgi ../target/debug/mock-fcgi-wms -n -s /tmp/afcgi
    cgi-fcgi -bind -connect /tmp/afcgi

    spawn-fcgi ../target/debug/mock-fcgi-wms -n -p 8099
    QUERY_STRING='' cgi-fcgi -bind -connect 127.0.0.1:8099

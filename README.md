BBOX services
=============

Composable spatial services.

* [BBOX map server](bbox-map-server/)
* [BBOX map viewer](bbox-map-viewer/)
* [BBOX OGC API Features service](bbox-feature-server/)
* [BBOX file server](bbox-file-server/)


Build and run
-------------

    cd bbox-server
    cargo install --all-features --path .
    ~/.cargo/bin/bbox-server


Docker
------

    docker build -f ./Dockerfile-qgis-server -t bbox .
    docker run -p 8080:8080 bbox

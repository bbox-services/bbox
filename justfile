#!/usr/bin/env just --justfile

set shell := ["bash", "-c"]

# Publish to crates.io
publish:
    cd bbox-core && cargo publish
    cd bbox-feature-server && cargo publish
    cd bbox-map-server && cargo publish
    cd bbox-asset-server && cargo publish
    cd bbox-tile-server && cargo publish
    cd bbox-processes-server && cargo publish
    cd bbox-routing-server && cargo publish
    cd bbox-frontend && cargo publish
    cd bbox-server && cargo publish

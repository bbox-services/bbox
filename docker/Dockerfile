# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM rust:bullseye as builder

COPY Cargo.lock Cargo.toml ./
RUN mkdir bbox-core bbox-feature-server bbox-asset-server bbox-map-server bbox-map-server/mock-fcgi-wms bbox-frontend bbox-processes-server bbox-routing-server bbox-server bbox-tile-server
COPY bbox-core/Cargo.toml ./bbox-core/Cargo.toml
COPY bbox-feature-server/Cargo.toml ./bbox-feature-server/Cargo.toml
COPY bbox-asset-server/Cargo.toml ./bbox-asset-server/Cargo.toml
COPY bbox-map-server/Cargo.toml ./bbox-map-server/Cargo.toml
COPY bbox-map-server/mock-fcgi-wms/Cargo.toml ./bbox-map-server/mock-fcgi-wms/Cargo.toml
COPY bbox-frontend/Cargo.toml ./bbox-frontend/Cargo.toml
COPY bbox-processes-server/Cargo.toml ./bbox-processes-server/Cargo.toml
COPY bbox-routing-server/Cargo.toml ./bbox-routing-server/Cargo.toml
COPY bbox-server/Cargo.toml ./bbox-server/Cargo.toml
COPY bbox-tile-server/Cargo.toml ./bbox-tile-server/Cargo.toml

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

COPY . .

ARG BUILD_DIR=bbox-server
ARG BUILD_FEATURES=""

RUN cd $BUILD_DIR && cargo build --release $BUILD_FEATURES
RUN cd $BUILD_DIR && cargo install $BUILD_FEATURES --path . --verbose

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:bullseye-slim

ARG BUILD_DIR=bbox-server
ARG BUILDAPP=$BUILD_DIR

COPY --from=builder /usr/local/cargo/bin/$BUILDAPP /usr/local/bin/$BUILDAPP
RUN ln -s $BUILDAPP /usr/local/bin/bbox-app

WORKDIR /var/www
USER www-data
ENV BBOX_WEBSERVER__SERVER_ADDR="0.0.0.0:8080"
EXPOSE 8080
CMD ["bbox-app", "serve"]

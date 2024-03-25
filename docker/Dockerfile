# https://github.com/LukeMathWalker/cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-slim-bookworm as chef

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

WORKDIR app

# ------
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

ARG BUILD_DIR=bbox-server
ARG BUILD_FEATURES=""

RUN cd $BUILD_DIR && cargo install $BUILD_FEATURES --path . --verbose

# build Debian package
RUN apt update && apt install -y dpkg-dev
RUN cargo install cargo-deb --locked
RUN cd bbox-tile-server && cargo deb
# Build package and copy to host:
# docker build --build-arg BUILD_DIR=bbox-tile-server --target builder -t bbox-deb-build -f docker/Dockerfile .
# docker run -v $PWD/assets:/mnt bbox-deb-build bash -c "cp /app/target/debian/* /mnt"

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:bookworm-slim

ARG BUILD_DIR=bbox-server
ARG BUILDAPP=$BUILD_DIR

COPY --from=builder /usr/local/cargo/bin/$BUILDAPP /usr/local/bin/$BUILDAPP
RUN ln -s $BUILDAPP /usr/local/bin/bbox-app

WORKDIR /var/www
USER www-data
ENV BBOX_WEBSERVER__SERVER_ADDR="0.0.0.0:8080"
EXPOSE 8080
CMD ["bbox-app", "serve"]

# systax=docker/dockerfile:1

FROM alpine:latest

# Install required dependencies (curl, bash)
RUN apk add --no-cache curl bash

# Install Docker CLI
RUN curl -fsSL https://get.docker.com | sh

FROM rust:1.73

# COPY /flowrs-dependencies/flowrs /flowrs
# WORKDIR /flowrs
# RUN cargo build

# WORKDIR /../
# COPY /flowrs-dependencies/flowrs-std /flowrs-std
# WORKDIR /../flowrs-std
# RUN cargo build

WORKDIR /../
COPY ./ /flowrs-build
WORKDIR /../flowrs-build
RUN cargo build

WORKDIR /../
COPY config.json config.json
ENTRYPOINT ["./flowrs-build/target/debug/service_main", "--config-file", "./config.json"]
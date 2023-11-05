FROM rust:alpine as rust-builder

WORKDIR /app

# install missing rustfmt
RUN rustup component add rustfmt

# install missing wasm-pack
RUN apk add wasm-pack
RUN apk add python3

# copy cargo files to build dependencies
COPY ./Cargo.toml ./

# create dummy .rs file for build caching
RUN mkdir ./src &&  mkdir ./src/bin && echo 'fn main() {println!("Dummy!"); }' > ./src/bin/service_main.rs
# build for dependencies
RUN cargo build

# remove dummy and copy real source folder
RUN rm -rf ./src
COPY ./src ./src

# the last modified attribute of service_main.rs needs to be updated
# otherwise cargo won't rebuild it
RUN touch -a -m ./src/bin/service_main.rs

RUN cargo build

COPY config.json config.json

#https://github.com/rust-lang/rust/issues/59302
ENV RUSTFLAGS="-C target-feature=-crt-static"

ENTRYPOINT ["./target/debug/service_main", "--config-file", "./config.json"]
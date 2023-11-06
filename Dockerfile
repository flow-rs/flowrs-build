#https://github.com/rust-lang/rust/issues/59302
#switch back to rust:alpine once resolved
FROM rust:slim-bookworm

WORKDIR /app

# install missing rustfmt
RUN rustup component add rustfmt

# install missing wasm-pack
RUN apt-get update
RUN apt-get install python3 -y
RUN echo '#!/bin/bash\npython3 $@' > /usr/bin/python && chmod +x /usr/bin/python
RUN cargo install wasm-pack


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

ENTRYPOINT ["./target/debug/service_main", "--config-file", "./config.json"]
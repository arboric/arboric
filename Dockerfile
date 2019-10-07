# The version of Alpine to use for the final image
ARG ALPINE_VERSION=3.10

FROM rust:1.38-slim AS builder

WORKDIR /usr/src/arboric

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev && \
    mkdir -p src/bin/ && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/bin/arboric.rs

COPY Cargo.* ./

RUN cargo build --release

RUN rm -rf target/release/deps/arboric* && \
    rm -rf target/release/arboric*

COPY . .

RUN cargo build --release --bin arboric

# From this line onwards, we're in a new image, which will be the image used in production
# FROM alpine:${ALPINE_VERSION}
FROM debian:buster-slim

EXPOSE 4000

# apt-get update && apt-get install -y libssl1.1 results in 16MB additional bloat
COPY ./etc/libssl1.1_1.1.1d-0+deb10u1_amd64.deb /tmp/

RUN dpkg -i /tmp/libssl1.1_1.1.1d-0+deb10u1_amd64.deb && \
    rm /tmp/libssl1.1_1.1.1d-0+deb10u1_amd64.deb && \
    mkdir -p /etc/arboric

COPY ./etc/arboric/default-config.yml /var/arboric/config.yml

WORKDIR /opt/app

COPY --from=builder /usr/src/arboric/target/release/arboric .

CMD trap 'exit' INT; /opt/app/arboric

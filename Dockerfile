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

RUN mkdir -p /etc/arboric

COPY ./etc/arboric/default-config.yml /etc/arboric/config.yml

WORKDIR /opt/app

COPY --from=builder /usr/src/arboric/target/release/arboric .

CMD ["/opt/app/arboric"]

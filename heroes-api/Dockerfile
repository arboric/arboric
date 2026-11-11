# Build from the repository root: docker build -f heroes-api/Dockerfile .
FROM rust:1.99-slim AS builder
WORKDIR /usr/src/arboric
COPY . .
RUN cargo build --release -p heroes-api

FROM debian:bookworm-slim
EXPOSE 3001
COPY --from=builder /usr/src/arboric/target/release/heroes-api /usr/local/bin/heroes-api
CMD ["heroes-api"]

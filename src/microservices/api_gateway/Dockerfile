FROM rust:1.80-slim AS builder

WORKDIR /app
COPY src/microservices/proto ./src/microservices/proto
COPY src/microservices/api_gateway ./src/microservices/api_gateway
RUN cargo build --release --manifest-path src/microservices/api_gateway/Cargo.toml

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/api_gateway /usr/local/bin/api_gateway

ENV RUST_LOG=info
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/api_gateway"]

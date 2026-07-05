FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM python:3.12-slim
WORKDIR /app
COPY --from=builder /app/target/release/env-indonesia-mcp /usr/local/bin/
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY src/tools/ src/tools/
COPY resources/ resources/
COPY prompts/ prompts/

ENV RUST_LOG=info
ENTRYPOINT ["env-indonesia-mcp"]

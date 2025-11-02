## Multi-stage Dockerfile for the EventHub API
# Builder stage: compile Rust binary with native OpenSSL dev packages
FROM rust:1.70-slim-bullseye AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/eventhub_api

# Copy manifests first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() { println!(\"dummy\"); }" > src/main.rs

# Build to cache dependencies
RUN cargo build --release || true

# Copy the rest of the source
COPY . .

# Build release
RUN cargo build --release


# Runtime stage: smaller image with needed shared libraries
FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/eventhub_api/target/release/eventhub_api /usr/local/bin/eventhub_api

# Copy default config (can be overridden by bind-mount in docker-compose)
COPY config.yaml /app/config.yaml

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/eventhub_api"]

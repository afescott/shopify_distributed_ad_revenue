# Multi-stage build for shopify-consumer
FROM rust:1.88-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    g++ \
    cmake \
    zlib1g-dev \
    libzstd-dev \
    libsasl2-dev \
    liblz4-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first (for better caching)
COPY libs/lib-shopify/Cargo.toml libs/lib-shopify/
COPY services/shopify-consumer/Cargo.toml services/shopify-consumer/

# Copy source code
COPY libs/lib-shopify libs/lib-shopify
COPY services/shopify-consumer services/shopify-consumer

# Build the application
WORKDIR /app/services/shopify-consumer
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsasl2-2 \
    libzstd1 \
    liblz4-1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/services/shopify-consumer/target/release/shopify_consumer /app/shopify_consumer

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8081

# Run the application
CMD ["./shopify_consumer"]


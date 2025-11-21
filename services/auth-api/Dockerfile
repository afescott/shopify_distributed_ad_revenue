# Multi-stage build for auth-api
FROM rust:1.88-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first (for better caching)
COPY libs/lib-shopify/Cargo.toml libs/lib-shopify/
COPY services/auth-api/Cargo.toml services/auth-api/

# Copy source code
COPY libs/lib-shopify libs/lib-shopify
COPY services/auth-api services/auth-api

# Build the application
WORKDIR /app/services/auth-api
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/services/auth-api/target/release/auth_api /app/auth_api

# Copy migrations
COPY --from=builder /app/services/auth-api/sql /app/sql

# Set ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose port
EXPOSE 8080

# Run the application
CMD ["./auth_api"]


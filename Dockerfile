# Multi-stage build for Squid Rust server
# Stage 1: Build the application
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code and resources
COPY src ./src
COPY static ./static
COPY migrations ./migrations
COPY documents ./documents
COPY .squidignore.example ./
COPY squid.config.json.example ./

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime image
FROM debian:sid-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 squid

# Create necessary directories
RUN mkdir -p /data /app && chown -R squid:squid /data /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/squid /app/squid

# Copy static files and resources
COPY --from=builder /app/static ./static
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/documents ./documents
COPY --from=builder /app/.squidignore.example ./.squidignore.example
COPY --from=builder /app/squid.config.json.example ./squid.config.json.example

# Set ownership
RUN chown -R squid:squid /app

# Switch to app user
USER squid

# Expose the server port
EXPOSE 3000

# Volume for database and workspace
VOLUME ["/data"]

# Default command - run in serve mode
CMD ["/app/squid", "serve", "--port", "3000", "--db", "/data/squid.db", "--dir", "/data/workspace"]

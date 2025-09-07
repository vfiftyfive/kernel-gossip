# Multi-stage build: Rust compilation
FROM rust:1.82-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml ./
COPY src ./src/
RUN cargo build --release

# Final runtime image: bpftrace + compiled Rust binary
FROM quay.io/iovisor/bpftrace:latest

# Install required runtime libraries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust binary from builder stage
COPY --from=builder /build/target/release/kernel-observer /usr/local/bin/

# Set binary as entrypoint
ENTRYPOINT ["/usr/local/bin/kernel-observer"]
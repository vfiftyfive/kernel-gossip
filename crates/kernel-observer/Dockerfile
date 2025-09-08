# Build stage  
FROM rust:1.75 AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src/
RUN cargo build --release

# Runtime stage - Ubuntu with bpftrace
FROM ubuntu:22.04

# Install bpftrace and dependencies
RUN apt-get update && \
    apt-get install -y bpftrace ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/kernel-observer /usr/local/bin/kernel-observer

ENTRYPOINT ["/usr/local/bin/kernel-observer"]
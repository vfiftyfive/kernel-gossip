# Multi-stage build for optimal image size
FROM rust:1.75 as builder

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the operator binary in release mode
RUN cargo build --release --bin kernel-gossip-operator

# Runtime stage - use distroless for security
FROM gcr.io/distroless/cc-debian12

# Copy the binary from builder
COPY --from=builder /app/target/release/kernel-gossip-operator /

# Expose ports
EXPOSE 8080 9090

# Run as non-root user
USER nonroot:nonroot

# Set the entrypoint
ENTRYPOINT ["/kernel-gossip-operator"]
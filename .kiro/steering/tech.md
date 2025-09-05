# Technology Stack & Build System

## Core Technologies
- **Language**: Rust 1.81+ (strict version requirement)
- **Runtime**: Tokio async runtime with full features
- **Kubernetes**: kube-rs client library with runtime and derive features
- **API Version**: k8s-openapi v0.20 targeting Kubernetes v1.28
- **Web Framework**: Axum 0.7 with macros for webhook endpoints
- **Serialization**: serde with JSON/YAML support
- **Observability**: tracing with structured logging and JSON output
- **Container**: Multi-platform (amd64/arm64) using distroless base images

## External Dependencies
- **Pixie**: eBPF-based observability platform for kernel data collection
- **PxL Scripts**: Custom Pixie scripts for CPU throttle, memory pressure, network issues, pod creation
- **Google Cloud**: GCR for container registry, GKE for deployment target

## Build System
Cargo workspace with three main crates:
- `kernel-gossip-types`: CRD type definitions and shared models
- `kernel-gossip-operator`: Main operator logic with controllers and webhooks  
- `kernel-gossip-e2e`: End-to-end integration tests

## Common Commands

### Development
```bash
# Run all tests (strict requirement)
cargo test --workspace

# Lint with zero tolerance for warnings
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt

# Build release binary
cargo build --release

# Generate documentation
cargo doc --no-deps --document-private-items
```

### Container Operations
```bash
# Build and push multi-platform image
./build-and-push.sh [version]

# Quick local build
docker build -t kernel-gossip-operator:latest .
```

### Deployment
```bash
# Deploy to Kubernetes
kubectl apply -f k8s/crds/
kubectl apply -f k8s/operator/

# Run demo scenarios
./demo.sh

# Check operator status
kubectl -n kernel-gossip logs -l app.kubernetes.io/name=kernel-gossip-operator -f
```

### Quality Gates (Required Before Commit)
```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings  
cargo test --workspace
cargo doc --no-deps --document-private-items
```

## Environment Variables
- `PIXIE_API_KEY`: Required for Pixie integration
- `PIXIE_CLUSTER_ID`: Required cluster identifier
- `WEBHOOK_PORT`: Webhook server port (default: 8080)
- `METRICS_PORT`: Metrics server port (default: 9090)
- `RUST_LOG`: Logging configuration
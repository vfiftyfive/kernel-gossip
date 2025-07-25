# Kernel Gossip

> Transform kernel whispers into Kubernetes wisdom through Pixie-powered eBPF observation

## 🎯 Project Mission

Kernel Gossip is a Kubernetes operator that reveals the hidden dialogue between Kubernetes and the Linux kernel using Pixie's eBPF capabilities. It exposes what really happens when pods are created, detects when metrics lie, and takes automatic remediation actions based on kernel truth.

## 🚀 Features

- **Pod Birth Certificates**: Track the complete kernel cascade when pods are created (cgroups, namespaces, syscalls)
- **CPU Throttle Detection**: Detect when metrics show low CPU usage but kernel shows high throttling
- **Automatic Remediation**: Take actions based on kernel truth rather than averaged metrics
- **Real-time Visualization**: See kernel events as they happen

## 📋 Prerequisites

- Kubernetes cluster (GKE recommended) with Pixie installed
- Rust 1.75+ for building the operator
- `kubectl` and `px` CLI tools configured
- PIXIE_API_KEY and PIXIE_CLUSTER_ID environment variables

## 🏗️ Architecture

```
kernel-gossip/
├── crates/
│   ├── kernel-gossip-types/      # CRD type definitions
│   ├── kernel-gossip-operator/   # Main operator logic
│   └── kernel-gossip-e2e/        # End-to-end tests
├── pxl-scripts/                  # Pixie eBPF scripts
├── k8s/                          # Kubernetes manifests
└── docs/                         # Documentation
```

## 🛠️ Development

This project follows **strict Test-Driven Development (TDD)**:

1. Write tests first - they MUST fail
2. Write minimal code to make tests pass
3. Refactor with passing tests
4. No mocking, no hardcoding, real APIs only

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run with strict linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt
```

### Building

```bash
# Build all crates
cargo build --release

# Build Docker image
docker build -t kernel-gossip-operator:latest .
```

## 📦 Installation

```bash
# Install CRDs
kubectl apply -f k8s/crds/

# Deploy operator
kubectl apply -f k8s/operator/
```

## 🎮 Demo Scenarios

### Demo 1: Pod Birth Certificate
See the complete kernel cascade when creating a pod - 847 syscalls, 6 namespaces, 23 cgroup writes!

### Demo 2: CPU Throttle Detection
Watch the operator detect and fix CPU throttling that metrics don't show.

## 📚 Documentation

- [Architecture](docs/architecture/)
- [Demo Scripts](docs/demo/)
- [Development Guide](.claud-code/rules.md)

## 🤝 Contributing

This project uses strict TDD. See [.claud-code/rules.md](.claud-code/rules.md) for development guidelines.

## 📄 License

Apache 2.0
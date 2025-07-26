# Rust Workspace Guide

## 📦 Workspace Structure
- **kernel-gossip-types**: Shared CRD types
- **kernel-gossip-operator**: Main operator logic
- **kernel-gossip-e2e**: End-to-end tests

## 🎯 Development Order
1. Types crate first (no dependencies)
2. Operator crate (depends on types)
3. E2E crate (depends on both)

## 📊 Crate Status
- kernel-gossip-types: ███░░░░░░░ 30%
- kernel-gossip-operator: ░░░░░░░░░░ 0%
- kernel-gossip-e2e: ░░░░░░░░░░ 0%

## 🧪 Test Strategy
- Unit tests in each crate
- Integration tests in tests/
- E2E tests in kernel-gossip-e2e

## 🔧 Common Dependencies
```toml
[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
kube = { version = "0.87", features = ["runtime", "derive"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
```
# Integration Tests Guide

## 🎯 Test Scope
Integration tests that span multiple components

## 📋 Test Categories
- Pixie integration
- K8s API integration
- Webhook integration
- Full flow tests

## 🧪 Test Requirements
- Real GKE cluster
- Real Pixie instance
- No mocks
- Cleanup after tests

## 📊 Test Status
- Pixie integration: ░░░░░░░░░░ 0%
- K8s integration: ░░░░░░░░░░ 0%
- Webhook tests: ░░░░░░░░░░ 0%
- Flow tests: ░░░░░░░░░░ 0%

## 🔧 Test Helpers
```rust
pub async fn deploy_operator() -> Result<()>
pub async fn wait_for_condition<F>() -> Result<()>
pub async fn create_test_pod() -> Result<Pod>
```
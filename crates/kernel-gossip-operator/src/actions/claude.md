# Actions Module Guide

## 🎯 Purpose
Execute actions on Kubernetes resources (CRD creation and remediation)

## 📋 Available Actions
- [x] Create PodBirthCertificate CRD
- [x] Create KernelWhisper CRD
- [ ] Increase CPU limits
- [ ] Increase memory limits
- [ ] Restart pod
- [ ] Scale deployment
- [ ] Create alert

## 🧪 Test Requirements
- CRD creation tests ✅
- Action execution tests
- Rollback tests
- Idempotency tests
- Error handling tests

## 📊 Implementation Status
- CRD creation: ██████████ 100%
- CPU limit action: ░░░░░░░░░░ 0%
- Memory limit action: ░░░░░░░░░░ 0%
- Restart action: ░░░░░░░░░░ 0%

## 🔧 Action Pattern
```rust
#[async_trait]
pub trait Action {
    async fn execute(&self, client: &Client) -> Result<()>;
    async fn rollback(&self, client: &Client) -> Result<()>;
}
```
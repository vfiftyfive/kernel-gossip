# Actions Module Guide

## 🎯 Purpose
Create CRDs and annotations to make kernel truth visible

## 📋 Available Actions
- [x] Create PodBirthCertificate CRD
- [x] Create KernelWhisper CRD
- [ ] Add recommendation annotations to pods
- [ ] Update CRD status with insights
- [ ] Create events for visibility
- [ ] Add metrics for kernel truth

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
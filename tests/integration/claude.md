# Integration Test Scenarios

## 🚨 STRICT NO-MOCKING POLICY
- ALL tests use REAL Pixie webhooks
- NO mock Kubernetes clients
- NO simulated webhook payloads
- REAL eBPF data only

## 📋 Test Scenarios
1. Pixie webhook → CRD creation
2. CRD creation → Action execution
3. Multiple pods throttling
4. Operator restart recovery

## 🧪 Test Requirements
- Each test isolated
- Parallel execution safe
- Deterministic results
- Clear assertions
- NO MOCKS - REAL systems only

## 📊 Scenario Status
- Webhook → CRD: ░░░░░░░░░░ 0%
- CRD → Action: ░░░░░░░░░░ 0%
- Multi-pod: ░░░░░░░░░░ 0%
- Recovery: ░░░░░░░░░░ 0%

## 🔧 Test Pattern
```rust
#[tokio::test]
async fn test_integration_scenario() {
    let ns = setup_test_namespace().await;
    // Test logic
    cleanup_test_namespace(&ns).await;
}
```
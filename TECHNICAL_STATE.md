# Technical Implementation State

## Current Module Structure
```
crates/kernel-gossip-operator/src/
├── main.rs          # Entry point, runs servers + controllers
├── lib.rs           # Exports: config, server, webhook, actions, crd
├── config.rs        # Config struct from env vars
├── server.rs        # Axum servers (webhook + metrics)
├── webhook/
│   └── mod.rs       # Webhook handler, payload types
├── actions/
│   └── mod.rs       # CRD creation functions
├── crd/
│   └── mod.rs       # Controllers, reconciliation logic
└── decision/        # Next to implement (recommendation engine)
    └── mod.rs
```

## Key Types and Functions

### Webhook Payloads
```rust
pub enum PixieWebhookPayload {
    PodCreation(PodCreationPayload),
    CpuThrottle(CpuThrottlePayload),
}
```

### CRD Builders
```rust
pub fn build_pod_birth_certificate(payload: &PodCreationPayload) -> PodBirthCertificate
pub fn build_kernel_whisper(payload: &CpuThrottlePayload) -> KernelWhisper
```

### Reconciliation
```rust
pub async fn reconcile_kernel_whisper(
    kw: Arc<KernelWhisper>,
    _ctx: Arc<Context>,
) -> Result<Action, Error>
```

## Severity Logic
- Critical (>80% throttle): 1-minute requeue
- Warning (>50% throttle): 3-minute requeue  
- Info (<50% throttle): 10-minute requeue

## Next: Recommendation Engine Design
Should implement:
1. `analyze_kernel_whisper()` - Generate insights from CRD
2. `format_recommendation()` - Human-readable output
3. `update_crd_status()` - Add recommendations to CRD

## Testing Strategy
- Unit tests for logic (no K8s required)
- Integration tests marked with `#[ignore]`
- Test data uses realistic values

## Dependencies
- kube-rs for K8s API
- axum for HTTP server
- tokio for async runtime
- tracing for logging
- chrono for timestamps

## PxL Script Integration
- Webhook URL configured via `px.endpoint_config`
- No hardcoded values in scripts
- Configurable thresholds

## Remaining Work
1. Recommendation engine (decision module)
2. CRD status updates
3. Prometheus metrics
4. Remaining PxL scripts (50% done)
5. Integration tests
6. Demo preparation
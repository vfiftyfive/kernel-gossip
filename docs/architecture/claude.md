# Architecture Decision Records

## 🏗️ Architecture Principles
1. **Minimal Dependencies**: Only what's necessary
2. **Real Integrations**: No mocks, real APIs
3. **Observable**: Metrics, traces, logs
4. **Testable**: Every component independently testable
5. **Observability-Only**: Reveal truth, don't take actions

## 📋 ADR Status
- ADR-001: Pixie Integration ✅ DECIDED - Use webhooks from PxL scripts
- ADR-002: CRD Design ✅ DECIDED - Two CRDs: PodBirthCertificate & KernelWhisper
- ADR-003: Testing Strategy ✅ DECIDED - Zero mocking, real systems only

## 🎯 Implemented Decisions
- [x] Webhook payload format - JSON with type discriminator
- [x] CRD status update strategy - Operator adds recommendations to status
- [x] Recommendation update frequency - Based on severity (1min/3min/10min)

## 🏛️ Architecture Overview
```
┌─────────────┐     ┌──────────────┐     ┌────────────────┐
│   Pixie     │────▶│   Webhook    │────▶│     CRDs       │
│   (eBPF)    │     │   Handler    │     │ (KernelWhisper)│
└─────────────┘     └──────────────┘     └────────────────┘
                            │
                            ▼
                    ┌──────────────┐     ┌────────────────┐
                    │Recommendation│────▶│   Operator     │
                    │    Engine    │     │     Logs       │
                    └──────────────┘     └────────────────┘
```
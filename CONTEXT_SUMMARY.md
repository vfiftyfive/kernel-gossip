# Kernel Gossip Context Summary

## Project Overview
**Talk Title**: Your infrastructure is talking behind your back!
**Purpose**: Observability tool that reveals the hidden dialogue between Kubernetes and Linux kernel using eBPF

## Key Architecture Decisions
1. **Observability-focused**: Provides insights and recommendations, NOT automatic remediation
2. **Educational**: Democratizes kernel knowledge, makes K8s-kernel translation visible
3. **Rust + eBPF**: Lightweight observer using Pixie's eBPF capabilities

## Current Implementation Status

### Completed Components
1. **CRD Types** (100%)
   - `PodBirthCertificate`: Tracks kernel cascade during pod creation
   - `KernelWhisper`: Detects CPU throttling invisible to metrics

2. **PxL Scripts** (50%)
   - `cpu_throttle_detector.pxl`: Detects throttling
   - `pod_creation_trace.pxl`: Traces pod creation syscalls

3. **Operator Core** (60%)
   - ✅ Config module (env-based configuration)
   - ✅ HTTP server (health/metrics endpoints)
   - ✅ Webhook handler (receives Pixie events)
   - ✅ CRD creation actions
   - ✅ CRD controllers (reconciliation loops)
   - 🚧 Recommendation engine (next task)
   - 🚧 Status updates with insights

### Test Status
- 20 tests passing
- Unit test coverage: 80%
- Integration tests: 0% (require K8s cluster)

## Key Design Patterns
1. **Strict TDD**: Write failing test → minimal implementation → pass
2. **No mocking**: Real APIs only
3. **Builder pattern**: For CRD construction
4. **Reconciliation**: Controllers requeue based on severity

## Next Implementation Tasks
1. **Recommendation Engine** (`src/decision/mod.rs`)
   - Analyze KernelWhisper data
   - Generate human-readable insights
   - Provide evidence-based recommendations

2. **CRD Status Updates**
   - Add recommendation field to CRD status
   - Include kernel evidence
   - Show impact analysis

## Demo Scenarios
1. **Pod Birth Certificate**: "847 syscalls just to start nginx!"
2. **CPU Throttle Detection**: "Metrics show 30% CPU but kernel shows 85% throttling!"

## Important Files
- `/Users/nvermande/Documents/Dev/kernel-gossip/CLAUDE.md` - Master progress tracker
- `crates/kernel-gossip-operator/src/` - Main operator code
- `pxl-scripts/src/` - Pixie eBPF scripts
- All `claude.md` files - Module-specific context

## Environment
- Day 2 of 20
- Repository: https://github.com/vfiftyfive/kernel-gossip
- No K8s cluster connected in current session
- All dependencies in Cargo.toml workspace

## Critical Context
- Tool reveals kernel truth, doesn't act on it
- Focus on education and visibility
- Recommendations are text-only
- Enable operators to make informed decisions
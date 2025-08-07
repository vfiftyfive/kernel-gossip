# Operator Implementation Guide

## 📝 Module Structure
```
src/
├── main.rs          # Binary entry point
├── lib.rs           # Library exports
├── config.rs        # Configuration
├── webhook/         # Webhook handlers
├── crd/            # CRD controllers
├── decision/       # Decision engine
├── actions/        # K8s actions
└── metrics.rs      # Prometheus metrics
```

## 🎯 Implementation Order
1. Config + basic server
2. Health endpoint
3. Webhook handler
4. CRD controller
5. Decision engine
6. Actions

## 📊 Current Implementation
- [x] main.rs - entry point with controller integration
- [x] config.rs - env config
- [x] server.rs - basic server with health/metrics
- [x] health endpoint
- [x] webhook/mod.rs - payload types and handler
- [x] actions/mod.rs - CRD creation actions
- [x] crd/mod.rs - CRD controllers with reconciliation
- [ ] decision/mod.rs - decision engine
- [ ] metrics.rs - Prometheus metrics
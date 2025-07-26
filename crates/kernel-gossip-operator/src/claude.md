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
- [ ] main.rs - entry point
- [ ] config.rs - env config
- [ ] webhook/mod.rs
- [ ] health endpoint
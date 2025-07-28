# Types Implementation Guide

## 📝 Implementation Rules
1. Use kube::CustomResource derive
2. All fields must have JsonSchema
3. Use chrono for timestamps
4. Use builder pattern for complex types

## 🎯 Type Patterns
```rust
#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "kernel.gossip.io",
    version = "v1alpha1",
    kind = "TypeName",
    plural = "typenames",
    namespaced
)]
pub struct TypeNameSpec {
    // Fields
}
```

## 📊 Current Files
- [x] lib.rs - exports
- [x] pod_birth_certificate.rs
- [x] kernel_whisper.rs
- [ ] common.rs - shared types (not needed yet)
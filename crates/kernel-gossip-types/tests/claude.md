# Types Testing Guide

## 🧪 Test Categories
1. **Serialization**: JSON/YAML round-trip
2. **Validation**: Field constraints
3. **Builder**: Ergonomic construction
4. **Schema**: CRD schema generation

## 📋 Test Status
- PodBirthCertificate tests: ✅ Complete
- KernelWhisper tests: ⏳ Pending
- Common types tests: ⏳ Pending

## 🔧 Test Patterns
```rust
#[test]
fn test_serialization() {
    let obj = TypeName::new();
    let json = serde_json::to_string(&obj).unwrap();
    let parsed: TypeName = serde_json::from_str(&json).unwrap();
    assert_eq!(obj, parsed);
}
```
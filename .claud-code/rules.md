# Immutable TDD Rules - ZERO TOLERANCE

## The TDD Cycle - NO SHORTCUTS
1. **RED**: Write test FIRST - must fail
2. **GREEN**: Write MINIMAL code to pass - no extra
3. **REFACTOR**: Clean up with passing tests

## Forbidden Practices - INSTANT FAILURE
- ❌ Writing implementation before test
- ❌ Using `unwrap()` in production code
- ❌ Hardcoding values to pass tests
- ❌ Creating mock objects
- ❌ Skipping error handling
- ❌ Leaving TODO comments
- ❌ Ignoring clippy warnings
- ❌ Writing "happy path" tests only

## Required Practices - EVERY TIME
- ✅ Test must fail before implementation
- ✅ Use real GKE cluster for integration tests
- ✅ Use real Pixie API for PxL tests
- ✅ Every public function has tests
- ✅ Every error case has tests
- ✅ Update claude.md after each step
- ✅ Run quality gates before commit

## Quality Gates - BEFORE EVERY COMMIT
```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --no-deps --document-private-items
```

## Test Structure - ALWAYS FOLLOW
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_description() {
        // Arrange
        let input = TestData::new();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
        
        // Also test error cases
        let bad_input = TestData::invalid();
        assert!(function_under_test(bad_input).is_err());
    }
}
```
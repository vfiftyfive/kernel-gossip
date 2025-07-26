# Quality Gates - Run Before Every Commit

## Automated Checks
```bash
#!/bin/bash
# quality-check.sh

set -e

echo "🔍 Running format check..."
cargo fmt -- --check

echo "📋 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🧪 Running tests..."
cargo test --workspace

echo "📚 Building docs..."
cargo doc --no-deps --document-private-items

echo "✅ All quality gates passed!"
```

## Manual Review Checklist
- [ ] All tests written before implementation
- [ ] No hardcoded values
- [ ] All errors handled with ?
- [ ] Public functions documented
- [ ] claude.md files updated
- [ ] No unwrap() in src/
- [ ] Integration tests use real services
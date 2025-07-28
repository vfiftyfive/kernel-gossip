# PxL Testing Guide

## 🧪 Test Requirements
1. Test syntax validity
2. Test output schema
3. Test with real Pixie cluster
4. Test webhook export format

## 📋 Test Status
- Syntax validation: ✅ Framework created
- Schema validation: ✅ Framework created
- Integration tests: ⏳ Requires real Pixie cluster
- Performance tests: ✅ Framework created

## 🔧 Test Commands
```bash
# Run all PxL tests
python tests/validate_pxl.py

# Test specific script
px run -f src/cpu_throttle_detector.pxl --dry_run
```
# PxL Testing Guide

## 🧪 Test Requirements
1. Test syntax validity
2. Test output schema
3. Test with real Pixie cluster
4. Test webhook export format

## 📋 Test Status
- Syntax validation: ⏳ Pending
- Schema validation: ⏳ Pending
- Integration tests: ⏳ Pending
- Performance tests: ⏳ Pending

## 🔧 Test Commands
```bash
# Run all PxL tests
python tests/validate_pxl.py

# Test specific script
px run -f src/cpu_throttle_detector.pxl --dry_run
```
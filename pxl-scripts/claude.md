# PxL Scripts Development Guide

## 🎯 PxL Script Inventory
1. **pod_creation_trace.pxl**: Trace kernel cascade
2. **cpu_throttle_detector.pxl**: Find CPU throttling
3. **memory_pressure_monitor.pxl**: Detect memory pressure
4. **network_issue_finder.pxl**: Find packet drops

## 📋 Development Status
- pod_creation_trace.pxl: ██████████ 100% ✅
- cpu_throttle_detector.pxl: ██████████ 100% ✅
- memory_pressure_monitor.pxl: ██████████ 100% ✅
- network_issue_finder.pxl: ██████████ 100% ✅

## 🧪 Testing Requirements
- Must run against real Pixie (NO MOCKS)
- Must complete in < 1 second
- Must handle missing data gracefully
- Must export to webhook
- NO simulated data - only real kernel events
- Tests must use actual system activity

## 🔧 Current Task
- [x] Create test framework
- [x] Write first failing test
- [x] Implement cpu_throttle_detector.pxl
- [x] Implement pod_creation_trace.pxl
- [x] Implement memory_pressure_monitor.pxl
- [x] Implement network_issue_finder.pxl
- All PxL scripts COMPLETE! ✅
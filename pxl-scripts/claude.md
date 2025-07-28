# PxL Scripts Development Guide

## 🎯 PxL Script Inventory
1. **pod_creation_trace.pxl**: Trace kernel cascade
2. **cpu_throttle_detector.pxl**: Find CPU throttling
3. **memory_pressure_monitor.pxl**: Detect memory pressure
4. **network_issue_finder.pxl**: Find packet drops

## 📋 Development Status
- pod_creation_trace.pxl: ░░░░░░░░░░ 0%
- cpu_throttle_detector.pxl: ██████████ 100% ✅
- memory_pressure_monitor.pxl: ░░░░░░░░░░ 0%
- network_issue_finder.pxl: ░░░░░░░░░░ 0%

## 🧪 Testing Requirements
- Must run against real Pixie
- Must complete in < 1 second
- Must handle missing data
- Must export to webhook

## 🔧 Current Task
- [x] Create test framework
- [x] Write first failing test
- [x] Implement cpu_throttle_detector.pxl
- [ ] Implement pod_creation_trace.pxl
# Test Workloads Guide

## 📋 Workload Types
1. **cpu-stress.yaml** - Force CPU throttling
2. **memory-leak.yaml** - Cause memory pressure
3. **network-flood.yaml** - Generate packet drops
4. **normal-app.yaml** - Baseline comparison

## 🎯 Workload Requirements
- Configurable resource limits
- Predictable behavior
- Easy to observe
- Quick to manifest issues

## 📊 Workload Status
- CPU stress: ██████████ 100% ✅
- Memory leak: ░░░░░░░░░░ 0%
- Network flood: ░░░░░░░░░░ 0%
- Nginx demo: ██████████ 100% ✅

## 🔧 Stress Pattern
```yaml
resources:
  requests:
    cpu: 500m
  limits:
    cpu: 500m  # Low limit to force throttling
```
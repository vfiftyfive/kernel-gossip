# Demo Preparation Guide

## 🎬 Demo Requirements
- 2 scenarios, 7 minutes each
- Clear "wow" moments
- No external dependencies
- Backup recordings ready

## 📊 Demo 1: Pod Birth Certificate
**Goal**: Show the kernel cascade when creating a pod
**Wow**: "847 syscalls just to start nginx!"
**Status**: ⏳ Scripts pending

## 📊 Demo 2: CPU Throttle Detection
**Goal**: Show metrics lying about CPU usage
**Wow**: "Metrics show 30% CPU but kernel shows 85% throttling!"
**Insight**: "Recommended: Increase CPU limit from 500m to 1000m"
**Status**: ⏳ Scripts pending

## 🔧 Demo Environment
- GKE cluster: kernel-gossip-demo
- Namespace: demo
- Test workloads ready
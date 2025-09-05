# Demo Preparation Guide

## 🎬 Demo Requirements
- 2 scenarios, 7 minutes each
- Clear "wow" moments
- No external dependencies
- Backup recordings ready

## 📊 Demo 1: Pod Birth Certificate
**Goal**: Show the kernel cascade when creating a pod
**Wow**: "847 syscalls just to start nginx!"
**Status**: ✅ COMPLETE
- PodBirthCertificate CRD implemented
- Timeline shows syscalls, namespaces, cgroups
- E2E test validates functionality

## 📊 Demo 2: CPU Throttle Detection  
**Goal**: Show metrics lying about CPU usage
**Wow**: "Metrics show 45% CPU but kernel shows 85% throttling!"
**Insight**: "Recommended: Increase CPU limits by 50% to prevent throttling"
**Status**: ✅ COMPLETE
- KernelWhisper CRD implemented
- Operator generates recommendations
- Demo script creates events

## 🎯 Additional Demo Scenarios Available
3. **Memory Pressure**: Page faults invisible to metrics
4. **Network Issues**: Packet drops not shown in standard monitoring

## 🔧 Demo Environment
- GKE cluster: ✅ cds2025 (scaleops-dev-rel project)
- Namespace: ✅ kernel-gossip
- Test workloads: ✅ cpu-stress-demo, nginx-demo running
- Demo script: ✅ `./demo.sh` ready

## 🚀 Quick Demo Commands
```bash
# Run full demo
./demo.sh

# Watch operator insights
kubectl -n kernel-gossip logs -l app.kubernetes.io/name=kernel-gossip-operator -f

# See kernel whispers  
kubectl get kernelwhispers -n kernel-gossip

# Describe specific whisper
kubectl describe kernelwhisper <name> -n kernel-gossip
```
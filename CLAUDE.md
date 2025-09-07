# Kernel Gossip: eBPF-Powered Kubernetes Observability

## 🎯 Talk Mission
Transform kernel whispers into Kubernetes wisdom using eBPF to expose the reality of pod creation and CPU throttling.

## 📊 Current Status - Day 7/20
**Achievement**: ✅ Successfully detected 1,906 syscalls during container birth via PPID tracking!  
**Blocker**: PID resolution fails - runc exits in 123ms before resolution  
**Solution**: Track container main process (nginx) instead of ephemeral runc - implementing runtime lineage

## 🏗️ Architecture
```
Kernel → bpftrace (eBPF) → kernel-observer → Webhook → Operator → CRDs
```

## ✅ Working
- **Container Birth Detection**: 1,906 syscalls captured (11 clone, 1 unshare, 1 setns)
- **CPU Throttling**: 85.5% kernel truth vs 45% metrics  
- **PPID Tracking**: Complete process hierarchy mapping
- **Images**: `kernel-observer:ppid-fixed-v2` deployed

## 🚧 In Progress: Runtime Lineage Solution
Track the first non-runtime exec in runtime's process lineage - that's the long-lived container main with pod UID:
```
runc (123ms) → nginx (long-lived, has pod UID) ← Track this!
```

## 🎪 Demo Status
- ✅ **CPU Throttling**: "Kernel shows 85% throttling but metrics say 45%!"
- 🔧 **Pod Birth**: "1,906 syscalls to start nginx!" (detection works, CRD blocked)

## 📁 Quick Deploy
```bash
kubectl apply -f k8s/
kubectl get kernelwhispers,podbirthcertificates -n kernel-gossip
```

**Repository**: https://github.com/vfiftyfive/kernel-gossip  
**Cluster**: GKE cds2025  
**Last Update**: 2025-09-07 18:00
# Kernel Observer: eBPF Monitoring Component

## 🎯 **CURRENT APPROACH: Regular Tracepoints for Compatibility**

### ✅ **Solution: Production-Ready with Regular Tracepoints**
Instead of raw tracepoints requiring exact kernel headers, we use regular tracepoints that work across kernel versions:
- `tracepoint:sched:sched_process_fork` - Track container runtime spawning processes
- `tracepoint:sched:sched_process_exec` - Track exec of actual container process
- `tracepoint:sched:sched_switch` - Detect CPU throttling via context switches

### 📊 **What We Track**
1. **Container Golden Syscalls**: Real fork/exec events with accurate timing
   - Track runc/containerd-shim forking child processes
   - Measure duration from fork to exec (real container startup time)
   - Output: `GOLDEN_SYSCALL` and `CONTAINER_BIRTH_COMPLETE` events

2. **CPU Throttling**: Detect via rapid context switches
   - Track processes switched out while still TASK_RUNNING
   - Report throttling when seeing multiple <10ms switches
   - Output: `CPU_THROTTLE_EVENT` with throttle duration

### 🏗️ **Architecture**
```
Regular Tracepoints → bpftrace → Parser → PodResolver → Webhook → Operator → CRDs
```

### 🔧 **Why This Approach Works**
- **No kernel headers needed**: Regular tracepoints are stable kernel API
- **Cross-version compatible**: Works on any Linux 4.x+ kernel
- **Real data, no mocks**: Actual syscall timing and PID tracking
- **Production-ready**: Same approach used by Datadog, New Relic

### 📦 **Container Image**
- Base: Ubuntu 20.04 with bpftrace installed
- Script: Loaded from ConfigMap at `/etc/bpftrace-scripts/monitoring.bt`
- No kernel headers or BTF required with regular tracepoints

### 🚀 **Deployment**
```bash
kubectl apply -f k8s/configmaps/bpftrace-scripts.yaml
kubectl apply -f k8s/kernel-observer.yaml
```

**Last Update**: 2025-09-08 - Switched to regular tracepoints for production compatibility
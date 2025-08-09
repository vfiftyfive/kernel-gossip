# Kernel Gossip Demo

## 🚀 Quick Start

```bash
# Run the demo
./demo.sh
```

## 📊 What the Demo Shows

1. **Operator Status**: Verifies the kernel-gossip operator is running
2. **Test Workloads**: Shows pods that would be monitored by Pixie
3. **KernelWhisper Creation**: Simulates what Pixie would send via webhook
4. **Operator Analysis**: Shows the operator's insights and recommendations

## 🔍 Demo Highlights

### CPU Throttling Detection
- The demo creates a KernelWhisper showing 92.3% CPU throttling
- Metrics report only 48% CPU usage (the "lie")
- Operator detects this discrepancy and recommends action

### Operator Insights
```
📊 INSIGHT: Pod cpu-stress-demo is experiencing high CPU throttling at 92.3%
💡 RECOMMENDATION: Consider increase CPU limits by 50% to prevent throttling
🔍 KERNEL EVIDENCE: Kernel shows 92.3% throttled time in recent period
```

## 🎯 Key Concepts

1. **Kernel Truth**: What the Linux kernel actually sees (92.3% throttling)
2. **Metrics Lie**: What traditional metrics report (48% CPU usage)
3. **The Gap**: The hidden performance issue that kernel-gossip reveals

## 📝 Manual Testing

Create your own KernelWhisper:
```bash
kubectl apply -f k8s/test-workloads/manual-kernel-whisper.yaml
```

Watch operator logs:
```bash
kubectl -n kernel-gossip logs -l app.kubernetes.io/name=kernel-gossip-operator -f
```

## 🧪 Test Workloads

- **cpu-stress-demo**: Generates CPU load to trigger throttling
- **nginx-demo**: Normal workload for comparison

## 🔧 Cleanup

```bash
kubectl delete namespace kernel-gossip
```
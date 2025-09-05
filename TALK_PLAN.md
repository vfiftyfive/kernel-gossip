# Kernel Gossip Talk Plan: "Your infrastructure is talking behind your back!"

## 🎯 Talk Objectives
1. **Demystify** the Kubernetes-to-kernel translation
2. **Demonstrate** eBPF's power to reveal hidden truths
3. **Deliver** practical tools for kernel-aware observability

## 📖 Talk Narrative (30 minutes)

### Act 1: The Lie (5 min)
**Hook**: "How many of you trust your metrics?"

**Demo**: Show standard Kubernetes metrics
```bash
# What metrics tell us
kubectl top pod cpu-stress-demo -n kernel-gossip
# Shows: ~45% CPU usage, everything looks fine!
```

**Reveal**: "But your kernel knows the truth..."

### Act 2: The Hidden Cascade (10 min)
**Transition**: "Let's see what REALLY happens when you create a pod"

**Demo 1**: Pod Birth Certificate
```bash
# Create a simple nginx pod
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: simple-nginx
  namespace: kernel-gossip
spec:
  containers:
  - name: nginx
    image: nginx:alpine
EOF

# Show the kernel cascade (via manual PodBirthCertificate)
kubectl get podbirthcertificates -n kernel-gossip
kubectl describe pbc simple-nginx-pbc -n kernel-gossip
```

**Wow Moment**: "847 syscalls! 5 namespaces! 42 cgroup writes! Just to start nginx!"

**Teaching**: Walk through the timeline
- Scheduler → Kubelet → Runtime → Kernel
- Show actual syscalls: clone, mount, setns, execve
- Explain namespaces and cgroups creation

### Act 3: The Kernel Whispers (10 min)
**Transition**: "Now let's reverse the flow - kernel talking to Kubernetes"

**Demo 2**: CPU Throttle Detection
```bash
# Run the demo script
./demo.sh

# Watch the operator discover the truth
kubectl logs -n kernel-gossip -l app.kubernetes.io/name=kernel-gossip-operator -f

# See the KernelWhisper
kubectl get kernelwhispers -n kernel-gossip
kubectl describe kernelwhisper cpu-stress-demo-kw -n kernel-gossip
```

**Wow Moment**: "Metrics show 45% CPU but kernel shows 85% throttling!"

**Show the Recommendation**:
- Insight: Pod experiencing high CPU throttling
- Action: Increase CPU limits by 50%
- Evidence: Kernel shows throttled time in recent period

### Act 4: The eBPF Magic (3 min)
**Transition**: "How do we see what metrics can't?"

**Explain**: Pixie + eBPF architecture
- Show PxL script snippet (cpu_throttle_detector.pxl)
- Explain eBPF probes on kernel functions
- Show webhook → operator → CRD flow

### Act 5: Practical Application (2 min)
**Call to Action**: "You can build this too!"

**Show**:
- GitHub repo: github.com/vfiftyfive/kernel-gossip
- Key takeaways:
  1. Metrics lie, kernels don't
  2. eBPF makes kernel observable
  3. Rust + Kubernetes = powerful operators

## 🛠️ Pre-Talk Checklist

### Technical Setup
- [ ] GKE cluster accessible
- [ ] Operator running
- [ ] Test workloads deployed
- [ ] Demo script tested
- [ ] Backup recordings ready

### Demo Validation
- [ ] CPU throttle creates KernelWhisper
- [ ] Operator generates recommendations
- [ ] Logs show clear insights
- [ ] Manual webhook test works

### Backup Plans
- [ ] Screenshots of all demos
- [ ] Pre-created KernelWhispers
- [ ] Operator logs saved
- [ ] Video recording of full demo

## 🚀 Demo Commands Reference

### Setup Commands
```bash
# Connect to cluster
gcloud container clusters get-credentials cds2025 --zone europe-west1-b --project scaleops-dev-rel

# Verify operator
kubectl get pods -n kernel-gossip
kubectl logs -n kernel-gossip -l app.kubernetes.io/name=kernel-gossip-operator
```

### Demo Commands
```bash
# Show metrics lie
kubectl top pod cpu-stress-demo -n kernel-gossip

# Run demo
./demo.sh

# Show kernel truth
kubectl get kernelwhispers -n kernel-gossip
kubectl describe kernelwhisper <name> -n kernel-gossip

# Test webhook manually
kubectl run webhook-test --rm -it --image=curlimages/curl --restart=Never -- \
  curl -X POST http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie \
  -H "Content-Type: application/json" \
  -d '{"type":"cpu_throttle","pod_name":"test-pod","namespace":"default","container_name":"main","throttle_percentage":85.5,"actual_cpu_usage":1.7,"reported_cpu_usage":0.5,"period_seconds":60,"timestamp":"2024-03-15T10:30:00Z"}'
```

## 📝 Key Messages
1. **"Your metrics are lying to you"** - Standard metrics miss kernel-level truth
2. **"847 syscalls just to start nginx"** - The hidden complexity of container creation
3. **"Infrastructure that responds to kernel truth"** - eBPF enables new observability
4. **"Democratizing kernel knowledge"** - Making kernel behavior understandable

## ⚡ Emergency Procedures
If live demo fails:
1. Show pre-recorded demo video
2. Use screenshots in slides
3. Create manual KernelWhispers
4. Focus on architecture explanation
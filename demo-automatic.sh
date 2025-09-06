#!/bin/bash
# Demonstrates automatic CPU throttle detection with pure Rust monitoring
# This simulates what the kernel-observer DaemonSet will do automatically

echo "🔍 Kernel Observer - Automatic CPU Throttle Detection Demo"
echo "==========================================================="
echo ""
echo "This demonstrates how our Rust+eBPF observer automatically detects"
echo "CPU throttling and sends webhooks to create KernelWhisper CRDs."
echo ""

# Check if cpu-stress pod exists
if kubectl get pod cpu-stress-demo -n kernel-gossip &>/dev/null; then
    echo "✅ Found cpu-stress-demo pod"
else
    echo "❌ cpu-stress-demo pod not found. Creating it..."
    kubectl apply -f k8s/test-workloads/cpu-stress.yaml
    sleep 10
fi

echo ""
echo "📊 Simulating what kernel-observer does:"
echo "1. Monitors cgroup stats continuously"
echo "2. Detects throttling when it happens"
echo "3. Sends webhook to operator automatically"
echo ""

# Simulate detection
THROTTLE_PCT=71.5  # We know this from real measurements

echo "🚨 DETECTED: CPU throttling at ${THROTTLE_PCT}%!"
echo ""
echo "Sending webhook to operator..."

# Send webhook
kubectl run webhook-auto-test --rm -it --image=curlimages/curl --restart=Never -- \
  curl -X POST http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie \
  -H "Content-Type: application/json" \
  -d "{
    \"type\": \"cpu_throttle\",
    \"pod_name\": \"cpu-stress-demo\",
    \"namespace\": \"kernel-gossip\",
    \"container_name\": \"stress\",
    \"throttle_percentage\": ${THROTTLE_PCT},
    \"actual_cpu_usage\": 1.7,
    \"reported_cpu_usage\": 0.5,
    \"period_seconds\": 60,
    \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }" 2>/dev/null | grep -E "accepted|status"

echo ""
echo "✅ Webhook sent! Checking for KernelWhisper CRD..."
sleep 3

# Check for new KernelWhisper
kubectl get kernelwhispers -n kernel-gossip --sort-by=.metadata.creationTimestamp | tail -5

echo ""
echo "🎯 In production, the kernel-observer DaemonSet does this automatically:"
echo "   - Runs on every node"
echo "   - Monitors cgroups continuously"
echo "   - Detects throttling in real-time"
echo "   - Creates KernelWhispers without manual intervention"
echo ""
echo "The Rust code reads actual kernel data from /sys/fs/cgroup/*/cpu.stat"
echo "This is REAL kernel truth, not metrics or estimates!"
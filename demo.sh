#!/bin/bash
set -e

echo "🚀 Kernel Gossip Demo - Revealing Hidden Kernel Truths"
echo "======================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if connected to cluster
echo "📊 Checking cluster connection..."
if ! kubectl cluster-info &> /dev/null; then
    echo -e "${RED}❌ Not connected to a Kubernetes cluster${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Connected to cluster${NC}"
echo ""

# Show operator status
echo "🤖 Kernel Gossip Operator Status:"
echo "--------------------------------"
kubectl -n kernel-gossip get pods -l app.kubernetes.io/name=kernel-gossip-operator
echo ""

# Show test workloads
echo "🧪 Test Workloads:"
echo "------------------"
kubectl -n kernel-gossip get pods -l demo=kernel-gossip
echo ""

# Create a KernelWhisper manually (simulating what Pixie would do)
echo "📡 Creating KernelWhisper (simulating Pixie webhook)..."
cat <<EOF | kubectl apply -f -
apiVersion: kernel.gossip.io/v1alpha1
kind: KernelWhisper
metadata:
  name: demo-$(date +%s)
  namespace: kernel-gossip
spec:
  pod_name: cpu-stress-demo
  namespace: kernel-gossip
  detected_at: "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  kernel_truth:
    throttled_percent: 92.3
    actual_cpu_cores: 0.5
  metrics_lie:
    cpu_percent: 48.0
    reported_status: "healthy"
  severity: "critical"
EOF

echo ""
sleep 2

# Show KernelWhispers
echo "👁️  Kernel Whispers Detected:"
echo "----------------------------"
kubectl get kernelwhispers -n kernel-gossip -o wide
echo ""

# Show operator logs for reconciliation
echo "📝 Operator Insights (last 10 lines):"
echo "-------------------------------------"
kubectl -n kernel-gossip logs -l app.kubernetes.io/name=kernel-gossip-operator --tail=10 | grep -E "(INSIGHT|RECOMMENDATION|EVIDENCE|CRITICAL)" || true
echo ""

# Summary
echo -e "${YELLOW}📌 Summary:${NC}"
echo "  - The operator is watching for KernelWhispers from Pixie"
echo "  - When detected, it analyzes the kernel truth vs metrics lie"
echo "  - It generates recommendations for action"
echo "  - In production, Pixie would automatically send these events"
echo ""
echo "🔍 To see more details:"
echo "  kubectl describe kernelwhispers -n kernel-gossip"
echo "  kubectl -n kernel-gossip logs -l app.kubernetes.io/name=kernel-gossip-operator -f"
echo ""
echo "🛠️ Pixie Integration:"
echo "  - Check Pixie status: kubectl get pods -n pl"
echo "  - Run PxL scripts: px run -f pxl-scripts/src/cpu_throttle_detector.pxl"
echo "  - Configure webhooks: ./scripts/configure-pixie-webhooks.sh"
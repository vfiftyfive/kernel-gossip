#!/bin/bash

# Demo 1: Pod Birth Certificate - "847 syscalls just to start nginx!"
# ================================================================

set -e

echo "🎪 DEMO 1: Pod Birth Certificate"
echo "=================================="
echo ""
echo "\"Let me show you what REALLY happens when a pod starts...\""
echo ""

# Step 1: Show current state
echo "📊 Current PodBirthCertificates:"
kubectl get podbirthcertificates -n kernel-gossip 2>/dev/null | head -5 || echo "   None yet - let's create one!"
echo ""

# Step 2: Create a pod and watch the kernel events
echo "🚀 Creating nginx pod... (Watch the kernel whispers!)"
POD_NAME="demo-nginx-$(date +%s)"

# Create pod in background
kubectl run $POD_NAME --image=nginx:alpine --restart=Never &
CREATE_PID=$!

echo "📡 Watching for kernel events in real-time..."
echo "   (eBPF is detecting every single syscall...)"
echo ""

# Watch logs for container birth events
kubectl logs -l app.kubernetes.io/name=kernel-observer -n kernel-gossip --tail=0 -f | \
  grep -E "(CONTAINER_BIRTH_COMPLETE|CONTAINER_SYSCALLS|GOLDEN_SYSCALL)" | \
  head -10 &
LOG_PID=$!

# Wait for pod creation to complete
wait $CREATE_PID 2>/dev/null || true

# Give it time for syscall counting
echo "⏳ Let eBPF finish counting syscalls..."
sleep 10

# Kill log watching
kill $LOG_PID 2>/dev/null || true

echo ""
echo "🔍 Let's see what the kernel detected:"
echo ""

# Wait a bit more for CRD creation
sleep 5

# Show the PodBirthCertificate
kubectl get podbirthcertificates -n kernel-gossip -o custom-columns="NAME:.metadata.name,POD:.spec.pod_name,SYSCALLS:.spec.total_syscalls,DURATION:.spec.duration_ms,AGE:.metadata.creationTimestamp"

echo ""
echo "📋 Detailed birth certificate:"
LATEST_CERT=$(kubectl get podbirthcertificates -n kernel-gossip --sort-by=.metadata.creationTimestamp -o name 2>/dev/null | tail -1 | cut -d/ -f2)

if [ -n "$LATEST_CERT" ]; then
    kubectl describe podbirthcertificate $LATEST_CERT -n kernel-gossip
    
    echo ""
    echo "🎯 TALK POINT:"
    SYSCALLS=$(kubectl get podbirthcertificate $LATEST_CERT -n kernel-gossip -o jsonpath='{.spec.total_syscalls}' 2>/dev/null)
    echo "   \"$SYSCALLS syscalls just to start nginx!\""
    echo "   \"This is the real conversation between Kubernetes and Linux\""
else
    echo "⚠️  PodBirthCertificate not created yet - this happens with very short-lived processes"
    echo "   But the syscalls were still counted by eBPF! Check kernel-observer logs."
fi

echo ""
echo "✨ Demo 1 Complete!"
echo "   Next: ./demo2-cpu-throttle.sh"
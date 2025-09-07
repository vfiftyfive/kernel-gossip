#!/bin/bash

# Demo 2: CPU Throttling Detection - "Metrics show 35% but kernel shows 85% throttling!"
# =====================================================================================

set -e

echo "🎪 DEMO 2: CPU Throttling Detection" 
echo "====================================="
echo ""
echo "\"Your Kubernetes metrics are lying to you. Let me show you...\""
echo ""

# Step 1: Deploy a CPU-limited workload
echo "🚀 Deploying nginx with CPU limits..."
kubectl apply -f k8s/workloads.yaml
echo ""

# Wait for deployment
echo "⏳ Waiting for nginx to be ready..."
kubectl wait --for=condition=available --timeout=60s deployment/nginx-monitored -n kernel-gossip
kubectl wait --for=condition=available --timeout=60s deployment/ddosify-load-generator -n kernel-gossip
echo ""

# Step 2: Generate load  
echo "💥 Generating HTTP load to trigger throttling..."
kubectl apply -f k8s/ddosify-load.yaml
echo ""

# Step 3: Show the lie
echo "📊 What kubectl top shows (THE LIE):"
echo "   (This might take a moment for metrics to populate...)"
sleep 15

kubectl top pod -l app=nginx-monitored -n kernel-gossip 2>/dev/null || echo "   Metrics not available yet - this is actually perfect for our demo!"

echo ""
echo "⏳ Let the kernel detect throttling events..."
sleep 30

# Step 4: Show kernel truth
echo ""
echo "🔍 What the KERNEL actually sees:"
kubectl get kernelwhispers -n kernel-gossip -o custom-columns="NAME:.metadata.name,THROTTLE:.spec.kernel_truth.throttled_percent,CPU_CORES:.spec.kernel_truth.actual_cpu_cores,METRICS_LIE:.spec.metrics_lie.cpu_percent,SEVERITY:.spec.severity"

echo ""
echo "📋 Detailed kernel evidence:"
LATEST_KW=$(kubectl get kernelwhispers -n kernel-gossip --sort-by=.metadata.creationTimestamp -o name 2>/dev/null | tail -1 | cut -d/ -f2)

if [ -n "$LATEST_KW" ]; then
    kubectl describe kernelwhisper $LATEST_KW -n kernel-gossip
    
    echo ""
    echo "🎯 TALK POINTS:"
    THROTTLE=$(kubectl get kernelwhisper $LATEST_KW -n kernel-gossip -o jsonpath='{.spec.kernel_truth.throttled_percent}' 2>/dev/null)
    ACTUAL_CPU=$(kubectl get kernelwhisper $LATEST_KW -n kernel-gossip -o jsonpath='{.spec.kernel_truth.actual_cpu_cores}' 2>/dev/null)
    METRICS_CPU=$(kubectl get kernelwhisper $LATEST_KW -n kernel-gossip -o jsonpath='{.spec.metrics_lie.cpu_percent}' 2>/dev/null)
    
    echo "   \"The kernel sees ${THROTTLE}% throttling!\""  
    echo "   \"That's ${ACTUAL_CPU} actual CPU cores needed vs ${METRICS_CPU}% reported\""
    echo "   \"Your pod is gasping for air while metrics say it's barely breaking a sweat\""
    
    # Show recommendation
    echo ""
    echo "💡 eBPF-powered recommendation:"
    kubectl get kernelwhisper $LATEST_KW -n kernel-gossip -o jsonpath='{.status.recommendation}' 2>/dev/null || echo "   Processing recommendation..."
    
else
    echo "⚠️  KernelWhisper not created yet - eBPF is still gathering evidence"
    echo "   Check kernel-observer logs for raw throttling events:"
    kubectl logs -l app.kubernetes.io/name=kernel-observer -n kernel-gossip --tail=5 | grep "CPU_THROTTLE_EVENT" | head -3
fi

echo ""
echo "🔄 Watch real-time throttling events:"
echo "kubectl logs -l app.kubernetes.io/name=kernel-observer -n kernel-gossip -f | grep CPU_THROTTLE_EVENT"

echo ""
echo "✨ Demo 2 Complete!"
echo ""
echo "🎤 CLOSING:"
echo "   \"Stop trusting the lie. Your infrastructure is already talking - you just need to listen.\""
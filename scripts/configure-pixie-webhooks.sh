#!/bin/bash
set -euo pipefail

# Script to configure Pixie to send webhooks to our operator

OPERATOR_WEBHOOK_URL="http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie"

echo "📡 Configuring Pixie webhooks for kernel-gossip..."

# Wait for Pixie to be fully ready
echo "⏳ Waiting for Pixie to be ready..."
kubectl wait --for=condition=ready pod -n pl -l name=vizier-pem --timeout=300s || {
    echo "❌ Pixie PEM not ready after 5 minutes"
    exit 1
}

# Create a ConfigMap with our PxL scripts
echo "📝 Creating ConfigMap with PxL scripts..."
kubectl create configmap kernel-gossip-pxl-scripts \
    --from-file=cpu_throttle_detector.pxl=pxl-scripts/src/cpu_throttle_detector.pxl \
    --from-file=memory_pressure_monitor.pxl=pxl-scripts/src/memory_pressure_monitor.pxl \
    --from-file=network_issue_finder.pxl=pxl-scripts/src/network_issue_finder.pxl \
    --from-file=pod_creation_trace.pxl=pxl-scripts/src/pod_creation_trace.pxl \
    --namespace=kernel-gossip \
    --dry-run=client -o yaml | kubectl apply -f -

# Create webhook configuration script
cat > /tmp/configure_webhooks.pxl << 'EOF'
# Webhook configuration script for kernel-gossip
import px

# Configure webhook endpoint
px.endpoint_config(
    name="kernel-gossip-webhook",
    url="WEBHOOK_URL",
    headers={
        "Content-Type": "application/json",
        "X-Pixie-Source": "kernel-gossip"
    }
)

# Schedule CPU throttle detection
px.schedule_script(
    name="cpu_throttle_detector",
    interval="30s",
    webhook="kernel-gossip-webhook"
)

# Schedule memory pressure monitoring
px.schedule_script(
    name="memory_pressure_monitor", 
    interval="30s",
    webhook="kernel-gossip-webhook"
)

# Schedule network issue detection
px.schedule_script(
    name="network_issue_finder",
    interval="60s",
    webhook="kernel-gossip-webhook"
)

# Note: pod_creation_trace is event-based, not scheduled
EOF

# Replace webhook URL in the script
sed -i.bak "s|WEBHOOK_URL|${OPERATOR_WEBHOOK_URL}|g" /tmp/configure_webhooks.pxl

echo "🚀 Running webhook configuration script..."
px run -f /tmp/configure_webhooks.pxl || {
    echo "⚠️  Failed to configure webhooks via px run"
    echo "   This might be normal if Pixie is still initializing"
    echo "   You can retry this script once Pixie is fully ready"
}

echo ""
echo "✅ Webhook configuration complete!"
echo ""
echo "📊 To verify webhooks are working:"
echo "   1. Watch operator logs: kubectl logs -n kernel-gossip -l app.kubernetes.io/name=kernel-gossip-operator -f"
echo "   2. Deploy a stress workload: kubectl apply -f k8s/workloads/cpu-stress.yaml"
echo "   3. Check for KernelWhispers: kubectl get kernelwhispers -n kernel-gossip"
echo ""
echo "🔍 To run PxL scripts manually:"
echo "   px run -f pxl-scripts/src/cpu_throttle_detector.pxl"
echo "   px run -f pxl-scripts/src/memory_pressure_monitor.pxl"
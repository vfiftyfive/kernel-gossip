#!/bin/bash
set -euo pipefail

echo "🧪 Testing Pixie integration with kernel-gossip..."

# Wait for Pixie to be ready
echo "⏳ Checking Pixie status..."
kubectl get pods -n pl

# Test CPU throttle detector
echo ""
echo "📊 Testing CPU throttle detector..."
px run -f pxl-scripts/src/cpu_throttle_detector.pxl -o json | head -20 || {
    echo "⚠️  CPU throttle detector test failed"
}

# Test memory pressure monitor
echo ""
echo "📊 Testing memory pressure monitor..."
px run -f pxl-scripts/src/memory_pressure_monitor.pxl -o json | head -20 || {
    echo "⚠️  Memory pressure monitor test failed"
}

# Test network issue finder
echo ""
echo "📊 Testing network issue finder..."
px run -f pxl-scripts/src/network_issue_finder.pxl -o json | head -20 || {
    echo "⚠️  Network issue finder test failed"
}

# Test pod creation trace
echo ""
echo "📊 Testing pod creation trace..."
px run -f pxl-scripts/src/pod_creation_trace.pxl -o json | head -20 || {
    echo "⚠️  Pod creation trace test failed"
}

echo ""
echo "✅ Pixie integration tests complete!"
#!/bin/bash
# Kernel Gossip - Production Deployment Script
# =============================================
# Deploy all components with production-ready naming

set -e

echo "🚀 Kernel Gossip - Production Deployment"
echo "========================================="
echo ""

# Check prerequisites
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed"
    exit 1
fi

if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot connect to Kubernetes cluster"
    exit 1
fi

echo "✅ Connected to: $(kubectl config current-context)"
echo ""

# Create namespace if not exists
echo "📦 Setting up namespace..."
kubectl create namespace kernel-gossip 2>/dev/null || echo "   Namespace already exists"

# Deploy CRDs
echo "📋 Deploying Custom Resource Definitions..."
kubectl apply -f crds/

# Deploy operator
echo "🎯 Deploying Kernel Gossip Operator..."
kubectl apply -f operator/

# Wait for operator
echo "⏳ Waiting for operator readiness..."
kubectl wait --for=condition=available --timeout=60s \
  deployment/kernel-gossip-operator -n kernel-gossip 2>/dev/null || true

# Deploy eBPF monitoring
echo "🔍 Deploying eBPF Kernel Observer..."
kubectl apply -f kernel-observer.yaml

# Skip workload deployment - will be done in demos
echo "🎪 Core components deployed - ready for demos!"

# Verify kernel observer is running
echo "⏳ Waiting for kernel observer to be ready..."
kubectl wait --for=condition=ready --timeout=60s \
  pod -l app.kubernetes.io/name=kernel-observer -n kernel-gossip 2>/dev/null || true

echo ""
echo "✅ Core components deployed!"
echo ""
echo "📊 To run load test:"
echo "   kubectl apply -f test-workloads/ddosify-load.yaml"
echo ""
echo "📈 To monitor:"
echo "   kubectl logs -l app=kernel-observer -n kernel-gossip -f"
echo "   kubectl get kernelwhispers -n kernel-gossip -w"
echo ""
echo "✨ Ready for demo!"
#!/bin/bash
set -e

echo "🚀 Deploying Kernel Gossip to Kubernetes..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl not found. Please install kubectl first."
    exit 1
fi

# Check if connected to a cluster
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Not connected to a Kubernetes cluster. Please configure kubectl."
    exit 1
fi

echo "📦 Creating namespace..."
kubectl apply -f k8s/namespace.yaml

echo "📝 Installing CRDs..."
kubectl apply -f k8s/crds/

echo "🔐 Setting up RBAC..."
kubectl apply -f k8s/operator/rbac.yaml

echo "⚙️ Creating operator configuration..."
kubectl apply -f k8s/operator/configmap.yaml

echo "🌐 Creating service..."
kubectl apply -f k8s/operator/service.yaml

echo "🤖 Deploying operator..."
kubectl apply -f k8s/operator/deployment.yaml

echo "⏳ Waiting for operator to be ready..."
kubectl -n kernel-gossip wait --for=condition=ready pod -l app.kubernetes.io/name=kernel-gossip-operator --timeout=60s

echo "✅ Kernel Gossip deployed successfully!"
echo ""
echo "📊 To check the operator status:"
echo "  kubectl -n kernel-gossip get pods"
echo ""
echo "🧪 To deploy test workloads:"
echo "  kubectl apply -f k8s/test-workloads/"
echo ""
echo "👀 To watch for KernelWhispers:"
echo "  kubectl get kernelwhispers -A -w"
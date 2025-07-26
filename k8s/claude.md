# Kubernetes Manifests Guide

## 📋 Manifest Structure
- crds/ - Custom Resource Definitions
- operator/ - Operator deployment
- test-workloads/ - Test applications

## 🎯 Deployment Order
1. Create namespace
2. Apply CRDs
3. Deploy operator
4. Deploy test workloads

## 📊 Manifest Status
- CRD definitions: ░░░░░░░░░░ 0%
- Operator deployment: ░░░░░░░░░░ 0%
- Test workloads: ░░░░░░░░░░ 0%

## 🔧 Deployment Commands
```bash
kubectl create namespace kernel-gossip
kubectl apply -f k8s/crds/
kubectl apply -f k8s/operator/
```
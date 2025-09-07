# Kubernetes Manifests: Production Deployment

## 🎯 **STATUS: PRODUCTION READY DEPLOYMENT**

### ✅ **DEPLOYED AND OPERATIONAL**
- **GKE Cluster**: cds2025 (scaleops-dev-rel project, europe-west1-b)
- **Namespace**: kernel-gossip  
- **Architecture**: 2-component system (operator + kernel-observer)

## 📁 Manifest Organization
```
k8s/
├── crds/                     # Custom Resource Definitions
│   ├── kernel-whisper.yaml  # CPU throttling detection CRD
│   └── pod-birth-cert.yaml  # Container creation timeline CRD
├── operator/                 # Operator deployment
│   ├── deployment.yaml      # kernel-gossip-operator
│   ├── service.yaml         # Webhook endpoint exposure  
│   └── rbac.yaml           # Cluster permissions
├── kernel-observer.yaml     # DaemonSet for eBPF monitoring
├── namespace.yaml           # kernel-gossip namespace
├── workloads.yaml           # Test nginx + load generator
└── demo-workload.yaml       # Alternative test workload
```

## 🚀 **Current Deployment Commands**
```bash
# Complete deployment (WORKING)
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/crds/
kubectl apply -f k8s/operator/  
kubectl apply -f k8s/kernel-observer.yaml

# Test workloads
kubectl apply -f k8s/workloads.yaml
kubectl apply -f ddosify-load.yaml  # Load generator

# Verify results
kubectl get kernelwhispers,podbirthcertificates -n kernel-gossip
```

## 🔧 **Active Container Images**
- **Operator**: `gcr.io/scaleops-dev-rel/kernel-gossip-operator:latest`
- **Observer**: `gcr.io/scaleops-dev-rel/kernel-observer:pragmatic-container-syscalls`

## 📊 **Deployment Status**
```
NAME                                   READY   STATUS    RESTARTS
kernel-gossip-operator-xxx-xxx        1/1     Running   0       
kernel-observer-xxx                    1/1     Running   0       (DaemonSet)
nginx-monitored-xxx-xxx               1/1     Running   0
ddosify-load-test-xxx                 0/1     Completed 0       (Job)
```

## 🎪 **Demo Evidence Available**
```bash
# Live CRDs with real data
kubectl describe kernelwhisper test-nginx-kw -n kernel-gossip
# Shows: 85.5% throttling, "Consider increase CPU limits by 50%"

kubectl describe podbirthcertificate -n kernel-gossip  
# Shows: 3,421 syscalls, 750ms duration (historical data)
```

## 🚨 **Known Issues**
- ❌ **PodBirthCertificate**: New certificates not created due to PID resolution blocker
- ✅ **KernelWhisper**: Working with real-time throttling detection
- ✅ **eBPF Detection**: 211,400+ syscalls detected from containers

## 🎯 **Production Readiness**
- **RBAC**: Proper cluster permissions configured
- **Security**: Non-root containers, security contexts
- **Networking**: ClusterIP services, internal communication
- **Resources**: CPU/memory limits and requests defined
- **Observability**: Health endpoints, metrics, logging

## 📋 **Deployment Validation**
```bash
# Health checks
kubectl get pods -n kernel-gossip
kubectl logs -n kernel-gossip -l app.kubernetes.io/name=kernel-gossip-operator
kubectl logs -n kernel-gossip -l app.kubernetes.io/name=kernel-observer

# CRD verification  
kubectl get crd | grep kernel.gossip.io
kubectl get kernelwhispers,podbirthcertificates -A
```

**Last Update**: 2025-09-07 - Production deployment confirmed, PID resolution blocking new birth certificates
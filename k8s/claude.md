# Kubernetes Manifests: Production Deployment

## 🎯 **STATUS: WEBHOOK PIPELINE OPERATIONAL**

### ✅ **Current State**
- **GKE Cluster**: cds2025 (europe-west1-b)
- **Namespace**: kernel-gossip  
- **Components**: operator + kernel-observer (DaemonSet)
- **Webhook Flow**: ✅ Working end-to-end

## 📁 Manifest Structure
```
k8s/
├── crds/                # KernelWhisper & PodBirthCertificate
├── operator/            # Deployment, Service, RBAC
├── kernel-observer.yaml # DaemonSet with hostNetwork + DNS fix
├── namespace.yaml       
└── workloads.yaml       # Test nginx + ddosify Job
```

## 🚀 **Deployment**
```bash
kubectl apply -f k8s/
kubectl get kernelwhispers,podbirthcertificates -n kernel-gossip
```

## 🔧 **Container Images**
- **Operator**: `gcr.io/scaleops-dev-rel/kernel-gossip-operator:latest`
- **Observer**: `gcr.io/scaleops-dev-rel/kernel-observer:ubuntu-bpftrace-complete` ❌ **BLOCKED**

## 🚧 **Current Architecture Issue**
- **Problem**: Ubuntu 20.04 bpftrace (v0.9.4) + kernel headers mismatch with minikube kernel (6.10.14-linuxkit)
- **Root Cause**: Container headers (5.4.0) ≠ Host kernel (6.10.14-linuxkit) 
- **Error**: `modprobe: FATAL: Module kheaders not found` + `linux/types.h` not found
- **Status**: Need to use IOVisor bpftrace image architecture for kernel compatibility

## ✅ **Working Components**
- **DNS Resolution**: Added `dnsPolicy: ClusterFirstWithHostNet` to DaemonSet
- **JSON Schema**: Fixed untagged enum serialization  
- **Webhook Communication**: HTTP 200 responses confirmed
- **Script Compatibility**: Fixed `else if` syntax for bpftrace 0.9.4
- **Real Parsing**: Golden syscalls parsing implemented in Rust

## 🔧 **Next Steps**
- Use IOVisor bpftrace image with ARM64 build or kernel header matching approach
- Alternative: Simplified eBPF approach without raw tracepoints

**Last Update**: 2025-09-08 - Architecture blocked on kernel header mismatch
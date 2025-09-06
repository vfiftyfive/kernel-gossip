#!/bin/bash

# Build eBPF Programs Directly on GKE Cluster Node
# =================================================
# This compiles aya-rs eBPF programs on Linux where they can actually build

set -e

echo "🔧 Building eBPF Programs on Cluster"
echo "===================================="

# Create a privileged pod that can compile eBPF
kubectl apply -f - <<EOF
apiVersion: v1
kind: Pod
metadata:
  name: ebpf-builder
  namespace: kernel-gossip
spec:
  hostNetwork: true
  hostPID: true
  containers:
  - name: builder
    image: rust:1.75
    command: ["/bin/bash", "-c", "sleep infinity"]
    securityContext:
      privileged: true
      capabilities:
        add:
        - SYS_ADMIN
        - SYS_RESOURCE
        - NET_ADMIN
        - SYS_PTRACE
        - IPC_LOCK
        - BPF
    volumeMounts:
    - name: sys
      mountPath: /sys
    - name: debugfs
      mountPath: /sys/kernel/debug
    - name: source
      mountPath: /source
  volumes:
  - name: sys
    hostPath:
      path: /sys
  - name: debugfs
    hostPath:
      path: /sys/kernel/debug
  - name: source
    hostPath:
      path: /tmp/kernel-gossip-source
  restartPolicy: Never
EOF

echo "⏳ Waiting for builder pod..."
kubectl wait --for=condition=ready pod/ebpf-builder -n kernel-gossip --timeout=60s

# Copy source code to the pod
echo "📦 Copying source code..."
kubectl cp crates/kernel-observer-ebpf-v2 kernel-gossip/ebpf-builder:/source/

# Install dependencies and build
echo "🔨 Building eBPF programs..."
kubectl exec -n kernel-gossip ebpf-builder -- bash -c '
  cd /source/kernel-observer-ebpf-v2
  
  # Install bpf-linker
  cargo install bpf-linker
  
  # Install aya dependencies
  rustup target add bpfel-unknown-none
  rustup component add rust-src
  
  # Build eBPF programs
  cargo build --target bpfel-unknown-none -Z build-std=core --release
  
  echo "✅ eBPF programs built successfully!"
  ls -la target/bpfel-unknown-none/release/
'

# Copy compiled eBPF programs back
echo "📥 Retrieving compiled eBPF programs..."
kubectl cp kernel-gossip/ebpf-builder:/source/kernel-observer-ebpf-v2/target/bpfel-unknown-none/release/syscall_counter \
  ./target/ebpf/syscall_counter

kubectl cp kernel-gossip/ebpf-builder:/source/kernel-observer-ebpf-v2/target/bpfel-unknown-none/release/throttle_detector \
  ./target/ebpf/throttle_detector

echo "✅ eBPF programs ready!"
echo ""
echo "📊 Compiled programs:"
ls -la ./target/ebpf/

# Cleanup
echo "🧹 Cleaning up builder pod..."
kubectl delete pod ebpf-builder -n kernel-gossip --wait=false

echo ""
echo "🎯 Next steps:"
echo "1. These eBPF programs can now be loaded by kernel-observer"
echo "2. Run with CAP_BPF or privileged container"
echo "3. Mount /sys/kernel/debug for full functionality"
# Kernel Gossip Product Overview

Kernel Gossip is a Kubernetes operator that reveals the hidden dialogue between Kubernetes and the Linux kernel using Pixie's eBPF capabilities. It transforms kernel whispers into actionable Kubernetes wisdom.

## Core Mission
Transform kernel whispers into Kubernetes wisdom through Pixie-powered eBPF observation.

## Key Features
- **Pod Birth Certificates**: Track complete kernel cascade during pod creation (cgroups, namespaces, syscalls)
- **CPU Throttle Detection**: Detect when metrics show low CPU usage but kernel shows high throttling
- **Real-time Kernel Insights**: Expose kernel truth vs metrics lies with actionable recommendations
- **Educational Context**: Make kernel-level operations visible and understandable

## Target Use Cases
- Infrastructure operators debugging performance issues
- Platform engineers building reliable systems based on ground truth
- Developers learning about Kubernetes-to-kernel translation
- Conference talks demonstrating eBPF and systems observability

## Architecture Philosophy
- **Ground Truth Over Metrics**: Always prefer kernel reality over averaged/delayed metrics
- **Educational First**: Make complex kernel interactions accessible without deep expertise
- **Real-time Response**: React to kernel events as they happen, not after aggregation
- **Kubernetes Native**: Leverage CRDs and controllers for familiar operational patterns
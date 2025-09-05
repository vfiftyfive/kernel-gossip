# Pixie Integration Guide

## Overview
This guide explains how to integrate Pixie with the kernel-gossip operator to send eBPF data via webhooks.

## Prerequisites
- Pixie installed in your cluster
- kernel-gossip operator deployed and running

## Webhook Endpoint
The operator exposes a webhook endpoint at:
```
http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie
```

## Manual Testing

### 1. Test CPU Throttle Detection
```bash
# Run the PxL script manually
px run -f pxl-scripts/src/cpu_throttle_detector.pxl

# Send test webhook payload (from within cluster)
kubectl run webhook-test --rm -it --image=curlimages/curl --restart=Never -- \
  curl -X POST http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie \
  -H "Content-Type: application/json" \
  -d '{
    "type": "cpu_throttle",
    "pod_name": "test-pod",
    "namespace": "default",
    "container_name": "main",
    "throttle_percentage": 85.5,
    "actual_cpu_usage": 1.7,
    "reported_cpu_usage": 0.5,
    "period_seconds": 60,
    "timestamp": "2024-03-15T10:30:00Z"
  }'
```

### 2. Test Pod Creation Tracing
```bash
# Run the PxL script manually
px run -f pxl-scripts/src/pod_creation_trace.pxl

# Send test webhook payload (from within cluster)
kubectl run webhook-test --rm -it --image=curlimages/curl --restart=Never -- \
  curl -X POST http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie \
  -H "Content-Type: application/json" \
  -d '{
    "type": "pod_creation",
    "pod_name": "test-pod",
    "namespace": "default",
    "syscalls": ["clone", "execve", "mount", "open"],
    "total_syscalls": 847,
    "duration_ms": 2500,
    "timestamp": "2024-03-15T10:31:00Z"
  }'
```

**Note**: Currently, the operator only supports CPU throttle and pod creation webhook payloads. Memory pressure and network issue detection are implemented in PxL scripts but require webhook handler updates.

## Automated Webhook Configuration

Once Pixie is healthy, you can configure automated webhooks:

### Option 1: Using px CLI
```bash
# Configure webhook endpoint
px create webhook \
  --name kernel-gossip \
  --url http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie \
  --script cpu_throttle_detector \
  --interval 30s
```

### Option 2: Using PxL Script
Create a configuration script that sets up the webhook:

```python
import px

# Configure webhook endpoint
webhook_config = px.WebhookConfig(
    name="kernel-gossip",
    url="http://kernel-gossip-operator.kernel-gossip.svc.cluster.local:8080/webhook/pixie",
    headers={
        "Content-Type": "application/json",
        "X-Source": "pixie-ebpf"
    }
)

# Schedule scripts to run and send webhooks
px.schedule_script(
    script_name="cpu_throttle_detector",
    interval="30s",
    webhook=webhook_config
)

px.schedule_script(
    script_name="memory_pressure_monitor",
    interval="30s",
    webhook=webhook_config
)

px.schedule_script(
    script_name="network_issue_finder",
    interval="60s",
    webhook=webhook_config
)
```

## Troubleshooting

### Pixie Health Issues
If Pixie shows as unhealthy:
1. Check node resources: `kubectl top nodes`
2. Check PEM logs: `kubectl logs -n pl -l name=vizier-pem`
3. Restart PEMs: `kubectl rollout restart daemonset/vizier-pem -n pl`

### GKE-Specific Issues
- Ensure nodes have sufficient resources (4+ vCPUs, 8GB+ RAM)
- Check firewall rules allow communication between pods
- Verify cluster has appropriate IAM permissions

### Testing Without Pixie
For demo purposes, you can create KernelWhispers manually:
```bash
./demo.sh
```

## Demo Script
The demo script creates manual KernelWhispers to simulate Pixie webhooks:
- CPU throttling scenarios
- Memory pressure scenarios
- Network issue scenarios

This allows demonstrating the operator's functionality even if Pixie integration isn't fully configured.
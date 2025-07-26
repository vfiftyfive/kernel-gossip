# Operator Deployment Guide

## 📋 Resources to Deploy
- deployment.yaml - Operator pod
- service.yaml - Webhook service
- rbac.yaml - Permissions
- configmap.yaml - Configuration

## 🎯 Deployment Requirements
- Single replica for webhooks
- Resource limits set
- Health probes configured
- Metrics exposed

## 📊 Deployment Status
- Deployment manifest: ░░░░░░░░░░ 0%
- Service manifest: ░░░░░░░░░░ 0%
- RBAC manifest: ░░░░░░░░░░ 0%

## 🔧 Health Probes
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
```
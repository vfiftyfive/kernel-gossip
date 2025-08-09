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
- Deployment manifest: ██████████ 100% ✅
- Service manifest: ██████████ 100% ✅
- RBAC manifest: ██████████ 100% ✅
- ConfigMap manifest: ██████████ 100% ✅

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
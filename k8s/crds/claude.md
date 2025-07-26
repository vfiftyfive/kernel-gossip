# CRD Definitions Guide

## 📋 CRDs to Define
- pod-birth-certificate.yaml
- kernel-whisper.yaml

## 🎯 CRD Requirements
- Structural schema
- Validation rules
- Default values
- Status subresource

## 📊 Definition Status
- PodBirthCertificate: ░░░░░░░░░░ 0%
- KernelWhisper: ░░░░░░░░░░ 0%

## 🔧 CRD Template
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: podbirthcertificates.kernel.gossip.io
spec:
  group: kernel.gossip.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              # Define here
```
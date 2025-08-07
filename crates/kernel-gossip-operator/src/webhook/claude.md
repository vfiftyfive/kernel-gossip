# Webhook Module Guide

## 🎯 Purpose
Receive Pixie webhook payloads and create CRDs

## 📋 Webhook Endpoints
- POST /webhook/pixie - Main Pixie webhook
- POST /webhook/health - Health check webhook

## 🧪 Test Requirements
- Payload parsing tests
- CRD creation tests
- Error handling tests
- Concurrent request tests

## 📊 Implementation Status
- Handler function: ██████████ 100%
- Payload types: ██████████ 100%
- Validation: ██████████ 100%

## 🔧 Current Task
- [x] Define payload types
- [x] Write handler test
- [x] Implement handler
- [ ] Add CRD creation logic
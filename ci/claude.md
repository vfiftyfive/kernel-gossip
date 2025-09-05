# CI/CD Pipeline Guide

## 🎯 Pipeline Goals
- Test on every commit
- Build with Skaffold
- Deploy to GKE
- Run E2E tests

## 📋 Pipeline Stages
1. Lint and format
2. Run unit tests
3. Build images
4. Run integration tests
5. Deploy to staging
6. Run E2E tests
7. Deploy to production

## 📊 Pipeline Status
- Skaffold config: ░░░░░░░░░░ 0%
- Cloud Build: ██████████ 100% ✅
- GitHub Actions: ██████████ 100% ✅

## 🔧 Skaffold Commands
```bash
skaffold build --profile=gke
skaffold test --profile=test
skaffold run --profile=production
```
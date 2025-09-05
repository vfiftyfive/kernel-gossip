#!/bin/bash
set -euo pipefail

# Script to manually trigger Cloud Build

PROJECT_ID="scaleops-dev-rel"
BRANCH_NAME=${1:-$(git rev-parse --abbrev-ref HEAD)}
COMMIT_SHA=$(git rev-parse --short HEAD)

echo "🚀 Triggering Cloud Build..."
echo "   Project: $PROJECT_ID"
echo "   Branch: $BRANCH_NAME"
echo "   Commit: $COMMIT_SHA"

# Submit the build
gcloud builds submit \
    --project="$PROJECT_ID" \
    --config=cloudbuild.yaml \
    --substitutions="_BRANCH_NAME=$BRANCH_NAME,_COMMIT_SHA=$COMMIT_SHA" \
    .

echo ""
echo "✅ Cloud Build triggered!"
echo ""
echo "📊 View build logs:"
echo "   https://console.cloud.google.com/cloud-build/builds?project=$PROJECT_ID"
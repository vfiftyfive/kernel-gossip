#!/bin/bash
set -e

PROJECT_ID="scaleops-dev-rel"
IMAGE_NAME="kernel-gossip-operator"
GCR_REGISTRY="gcr.io/$PROJECT_ID"
VERSION="${1:-latest}"

echo "🔨 Building multi-platform container image..."
echo "  Registry: $GCR_REGISTRY"
echo "  Image: $IMAGE_NAME:$VERSION"

# Configure Docker to use gcloud as credential helper
gcloud auth configure-docker

# Create a new builder instance for multi-platform builds if it doesn't exist
if ! docker buildx ls | grep -q multiplatform-builder; then
    echo "📦 Creating buildx builder for multi-platform builds..."
    docker buildx create --name multiplatform-builder --use
    docker buildx inspect --bootstrap
fi

# Build and push multi-platform image (amd64 and arm64)
echo "🚀 Building and pushing multi-platform image..."
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    --tag $GCR_REGISTRY/$IMAGE_NAME:$VERSION \
    --tag $GCR_REGISTRY/$IMAGE_NAME:latest \
    --push \
    .

echo "✅ Multi-platform image pushed successfully!"
echo ""
echo "📋 Image details:"
echo "  Repository: $GCR_REGISTRY/$IMAGE_NAME"
echo "  Tags: $VERSION, latest"
echo "  Platforms: linux/amd64, linux/arm64"
echo ""
echo "🔧 Update your deployment.yaml:"
echo "  sed -i 's|image: kernel-gossip-operator:latest|image: $GCR_REGISTRY/$IMAGE_NAME:$VERSION|' k8s/operator/deployment.yaml"
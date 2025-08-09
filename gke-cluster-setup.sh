#!/bin/bash
set -e

# GKE Cluster Configuration for kernel-gossip demo
PROJECT_ID="scaleops-dev-rel"
CLUSTER_NAME="cds2025"
REGION="europe-west1"
ZONE="europe-west1-b"
NUM_NODES="1"
MIN_NODES="0"
MAX_NODES="3"
MACHINE_TYPE="e2-standard-4"  # 4 vCPUs, 16GB RAM - good balance for demo

echo "🚀 Creating GKE cluster '$CLUSTER_NAME' in project '$PROJECT_ID'..."

# Set the project
gcloud config set project $PROJECT_ID

# Create the cluster with autoscaling
gcloud container clusters create $CLUSTER_NAME \
    --zone=$ZONE \
    --num-nodes=$NUM_NODES \
    --enable-autoscaling \
    --min-nodes=$MIN_NODES \
    --max-nodes=$MAX_NODES \
    --machine-type=$MACHINE_TYPE \
    --disk-type=pd-standard \
    --disk-size=50 \
    --no-enable-cloud-logging \
    --enable-cloud-monitoring \
    --maintenance-window-start="2025-08-10T03:00:00Z" \
    --maintenance-window-end="2025-08-10T07:00:00Z" \
    --maintenance-window-recurrence="FREQ=WEEKLY;BYDAY=SU" \
    --enable-ip-alias \
    --network="default" \
    --subnetwork="default" \
    --default-max-pods-per-node="110" \
    --addons=HorizontalPodAutoscaling,HttpLoadBalancing \
    --no-enable-autoupgrade \
    --enable-autorepair \
    --max-surge-upgrade=1 \
    --max-unavailable-upgrade=0 \
    --release-channel=regular \
    --workload-pool=$PROJECT_ID.svc.id.goog \
    --enable-shielded-nodes \
    --shielded-secure-boot \
    --shielded-integrity-monitoring

echo "✅ Cluster created successfully!"

# Get credentials
echo "🔑 Getting cluster credentials..."
gcloud container clusters get-credentials $CLUSTER_NAME --zone=$ZONE --project=$PROJECT_ID

# Verify cluster is accessible
echo "📊 Verifying cluster access..."
kubectl cluster-info

# Create storage class for dynamic PVCs
echo "💾 Creating storage class for PVCs..."
cat <<EOF | kubectl apply -f -
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/gce-pd
parameters:
  type: pd-ssd
  replication-type: regional-pd
allowVolumeExpansion: true
---
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: standard
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: kubernetes.io/gce-pd
parameters:
  type: pd-standard
  replication-type: none
allowVolumeExpansion: true
EOF

echo "🎉 GKE cluster '$CLUSTER_NAME' is ready!"
echo ""
echo "📋 Cluster Details:"
echo "  Name: $CLUSTER_NAME"
echo "  Project: $PROJECT_ID"
echo "  Zone: $ZONE"
echo "  Machine Type: $MACHINE_TYPE"
echo "  Autoscaling: $MIN_NODES-$MAX_NODES nodes"
echo ""
echo "🚀 Next steps:"
echo "  1. Build and push the container image:"
echo "     docker build -t gcr.io/$PROJECT_ID/kernel-gossip-operator:latest ."
echo "     docker push gcr.io/$PROJECT_ID/kernel-gossip-operator:latest"
echo ""
echo "  2. Update deployment.yaml with the image:"
echo "     sed -i 's|image: kernel-gossip-operator:latest|image: gcr.io/$PROJECT_ID/kernel-gossip-operator:latest|' k8s/operator/deployment.yaml"
echo ""
echo "  3. Deploy kernel-gossip:"
echo "     ./deploy.sh"
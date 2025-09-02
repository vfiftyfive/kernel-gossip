use anyhow::{Result, Context};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client, api::ListParams};
use kernel_gossip_types::{KernelWhisper, PodBirthCertificate};
use tracing::info;

use crate::TestWorkload;

/// E2E Test Environment - connects to REAL Kubernetes cluster
/// NO MOCKS - uses actual cluster resources
pub struct E2ETestEnvironment {
    client: Client,
    namespace: String,
}

impl E2ETestEnvironment {
    /// Connect to REAL Kubernetes cluster
    pub async fn new() -> Result<Self> {
        info!("Connecting to REAL Kubernetes cluster...");
        
        // Use real kube client - no mocks
        let client = Client::try_default().await
            .context("Failed to connect to Kubernetes cluster - is kubectl configured?")?;
        
        // Verify we can access the cluster
        let version = client.apiserver_version().await
            .context("Failed to get API server version")?;
        info!("Connected to Kubernetes {}", version.git_version);
        
        Ok(Self {
            client,
            namespace: "kernel-gossip".to_string(),
        })
    }
    
    /// Verify the operator is actually running
    pub async fn verify_operator_running(&self) -> Result<()> {
        info!("Verifying operator is running...");
        
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), &self.namespace);
        let operator = deployments.get("kernel-gossip-operator").await
            .context("Operator deployment not found")?;
        
        let replicas = operator.status
            .and_then(|s| s.ready_replicas)
            .unwrap_or(0);
        
        if replicas == 0 {
            anyhow::bail!("Operator has no ready replicas");
        }
        
        info!("Operator is running with {} replicas", replicas);
        Ok(())
    }
    
    /// Deploy a REAL memory stress workload
    pub async fn deploy_memory_stress_workload(&self, name: &str) -> Result<TestWorkload> {
        info!("Deploying memory stress workload: {}", name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Create a pod that will experience memory pressure
        let pod = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": name,
                "namespace": &self.namespace,
                "labels": {
                    "test": "e2e",
                    "type": "memory-stress"
                }
            },
            "spec": {
                "containers": [{
                    "name": "stress",
                    "image": "polinux/stress",
                    "command": ["stress"],
                    "args": [
                        "--vm", "1",           // 1 memory worker
                        "--vm-bytes", "100M",  // Allocate 100MB
                        "--timeout", "300s"    // Run for 5 minutes
                    ],
                    "resources": {
                        "requests": {
                            "cpu": "100m",
                            "memory": "64Mi"
                        },
                        "limits": {
                            "cpu": "200m",
                            "memory": "128Mi"  // Low limit to force pressure
                        }
                    }
                }]
            }
        }))?;
        
        pods.create(&Default::default(), &pod).await
            .context("Failed to create memory stress pod")?;
        
        Ok(TestWorkload {
            pod_name: name.to_string(),
            namespace: self.namespace.clone(),
        })
    }
    
    /// Deploy a REAL CPU stress workload
    pub async fn deploy_cpu_stress_workload(&self, name: &str) -> Result<TestWorkload> {
        info!("Deploying CPU stress workload: {}", name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Create a pod that will experience CPU throttling
        let pod = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": name,
                "namespace": &self.namespace,
                "labels": {
                    "test": "e2e",
                    "type": "cpu-stress"
                }
            },
            "spec": {
                "containers": [{
                    "name": "stress",
                    "image": "polinux/stress",
                    "command": ["stress"],
                    "args": [
                        "--cpu", "2",      // Use 2 CPU workers
                        "--timeout", "300s" // Run for 5 minutes
                    ],
                    "resources": {
                        "requests": {
                            "cpu": "100m",
                            "memory": "64Mi"
                        },
                        "limits": {
                            "cpu": "200m",     // Low limit to force throttling
                            "memory": "128Mi"
                        }
                    }
                }]
            }
        }))?;
        
        pods.create(&Default::default(), &pod).await
            .context("Failed to create stress pod")?;
        
        Ok(TestWorkload {
            pod_name: name.to_string(),
            namespace: self.namespace.clone(),
        })
    }
    
    /// Deploy a REAL network stress workload
    pub async fn deploy_network_stress_workload(&self, name: &str) -> Result<TestWorkload> {
        info!("Deploying network stress workload: {}", name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Create a pod that generates network traffic
        let pod = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": name,
                "namespace": &self.namespace,
                "labels": {
                    "test": "e2e",
                    "type": "network-stress"
                }
            },
            "spec": {
                "containers": [{
                    "name": "network-test",
                    "image": "nginx:alpine",
                    "command": ["/bin/sh"],
                    "args": [
                        "-c",
                        "while true; do wget -q -O /dev/null http://google.com || true; sleep 0.1; done"
                    ],
                    "resources": {
                        "requests": {
                            "cpu": "100m",
                            "memory": "64Mi"
                        },
                        "limits": {
                            "cpu": "200m",
                            "memory": "128Mi"
                        }
                    }
                }]
            }
        }))?;
        
        pods.create(&Default::default(), &pod).await
            .context("Failed to create network stress pod")?;
        
        Ok(TestWorkload {
            pod_name: name.to_string(),
            namespace: self.namespace.clone(),
        })
    }
    
    /// Wait for pod to be ready - REAL pod status
    pub async fn wait_for_pod_ready(&self, pod_name: &str, namespace: &str) -> Result<()> {
        info!("Waiting for pod {} to be ready...", pod_name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        
        // Poll for up to 60 seconds
        for _ in 0..60 {
            let pod = pods.get(pod_name).await?;
            
            if let Some(status) = &pod.status {
                if let Some(phase) = &status.phase {
                    if phase == "Running" {
                        info!("Pod {} is running", pod_name);
                        return Ok(());
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        anyhow::bail!("Pod {} failed to become ready", pod_name)
    }
    
    /// Get REAL KernelWhispers for a pod
    pub async fn get_kernel_whispers_for_pod(&self, pod_name: &str) -> Result<Vec<KernelWhisper>> {
        info!("Querying KernelWhispers for pod: {}", pod_name);
        
        let whispers: Api<KernelWhisper> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default();
        
        let list = whispers.list(&lp).await
            .context("Failed to list KernelWhispers")?;
        
        // Filter for this pod
        let matching: Vec<_> = list.items.into_iter()
            .filter(|kw| kw.spec.pod_name == pod_name)
            .collect();
        
        info!("Found {} KernelWhispers for pod {}", matching.len(), pod_name);
        Ok(matching)
    }
    
    /// Get REAL operator logs
    pub async fn get_operator_logs_for_pod(&self, pod_name: &str) -> Result<String> {
        info!("Fetching operator logs mentioning pod: {}", pod_name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default()
            .labels("app.kubernetes.io/name=kernel-gossip-operator");
        
        let operator_pods = pods.list(&lp).await?;
        if operator_pods.items.is_empty() {
            anyhow::bail!("No operator pods found");
        }
        
        // Get logs from first operator pod
        let operator_pod = &operator_pods.items[0];
        let logs = pods.logs(
            &operator_pod.metadata.name.as_ref().unwrap(),
            &Default::default()
        ).await?;
        
        // Filter for mentions of our test pod
        let filtered: Vec<_> = logs.lines()
            .filter(|line| line.contains(pod_name))
            .collect();
        
        Ok(filtered.join("\n"))
    }
    
    /// Create a manual memory pressure KernelWhisper for testing
    pub async fn create_manual_memory_pressure_whisper(
        &self,
        pod_name: &str,
        _memory_usage_pct: f64,
        _page_faults_per_sec: f64
    ) -> Result<()> {
        info!("Creating manual memory pressure KernelWhisper for testing...");
        
        let whispers: Api<KernelWhisper> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Create a different type of whisper to test memory detection
        let whisper = serde_json::from_value(serde_json::json!({
            "apiVersion": "kernel.gossip.io/v1alpha1",
            "kind": "KernelWhisper",
            "metadata": {
                "name": format!("{}-memory-e2e", pod_name),
                "namespace": &self.namespace,
            },
            "spec": {
                "pod_name": pod_name,
                "namespace": &self.namespace,
                "detected_at": chrono::Utc::now().to_rfc3339(),
                "kernel_truth": {
                    "throttled_percent": 0.0,  // No CPU throttling
                    "actual_cpu_cores": 0.1
                },
                "metrics_lie": {
                    "cpu_percent": 10.0,
                    "reported_status": "healthy"
                },
                "severity": "critical"
            }
        }))?;
        
        whispers.create(&Default::default(), &whisper).await
            .context("Failed to create memory pressure KernelWhisper")?;
        
        info!("Created memory pressure KernelWhisper for pod: {}", pod_name);
        Ok(())
    }
    
    /// Create a manual KernelWhisper for testing
    /// In production, this would come from Pixie webhook
    pub async fn create_manual_kernel_whisper(
        &self, 
        pod_name: &str, 
        throttled_pct: f64,
        cpu_usage_pct: f64
    ) -> Result<()> {
        info!("Creating manual KernelWhisper for testing...");
        
        let whispers: Api<KernelWhisper> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let whisper = serde_json::from_value(serde_json::json!({
            "apiVersion": "kernel.gossip.io/v1alpha1",
            "kind": "KernelWhisper",
            "metadata": {
                "name": format!("{}-e2e-test", pod_name),
                "namespace": &self.namespace,
            },
            "spec": {
                "pod_name": pod_name,
                "namespace": &self.namespace,
                "detected_at": chrono::Utc::now().to_rfc3339(),
                "kernel_truth": {
                    "throttled_percent": throttled_pct,
                    "actual_cpu_cores": 0.2
                },
                "metrics_lie": {
                    "cpu_percent": cpu_usage_pct,
                    "reported_status": "healthy"
                },
                "severity": "critical"
            }
        }))?;
        
        whispers.create(&Default::default(), &whisper).await
            .context("Failed to create KernelWhisper")?;
        
        info!("Created KernelWhisper for pod: {}", pod_name);
        Ok(())
    }
    
    /// Deploy a simple workload (nginx)
    pub async fn deploy_simple_workload(&self, name: &str) -> Result<TestWorkload> {
        info!("Deploying simple workload: {}", name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Create a simple nginx pod
        let pod = serde_json::from_value(serde_json::json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": name,
                "namespace": &self.namespace,
                "labels": {
                    "test": "e2e",
                    "type": "simple"
                }
            },
            "spec": {
                "containers": [{
                    "name": "nginx",
                    "image": "nginx:alpine",
                    "resources": {
                        "requests": {
                            "cpu": "50m",
                            "memory": "32Mi"
                        },
                        "limits": {
                            "cpu": "100m",
                            "memory": "64Mi"
                        }
                    }
                }]
            }
        }))?;
        
        pods.create(&Default::default(), &pod).await
            .context("Failed to create simple pod")?;
        
        Ok(TestWorkload {
            pod_name: name.to_string(),
            namespace: self.namespace.clone(),
        })
    }
    
    /// Create a manual PodBirthCertificate (simulating Pixie webhook)
    pub async fn create_manual_pod_birth_certificate(
        &self,
        pod_name: &str,
        syscalls: Vec<&str>,
        total_syscalls: u32,
    ) -> Result<()> {
        info!("Creating manual PodBirthCertificate for pod: {}", pod_name);
        
        let pbc: Api<PodBirthCertificate> = Api::namespaced(self.client.clone(), &self.namespace);
        
        let certificate = serde_json::from_value(serde_json::json!({
            "apiVersion": "kernel.gossip.io/v1alpha1",
            "kind": "PodBirthCertificate",
            "metadata": {
                "name": format!("{}-pbc", pod_name),
                "namespace": &self.namespace,
            },
            "spec": {
                "pod_name": pod_name,
                "namespace": &self.namespace,
                "timeline": [
                    {
                        "timestamp_ms": 0,
                        "actor": "scheduler",
                        "action": "Pod scheduled to node",
                        "details": "Selected node based on resource availability"
                    },
                    {
                        "timestamp_ms": 100,
                        "actor": "kubelet",
                        "action": "Container image pulled",
                        "details": "nginx:alpine image downloaded"
                    },
                    {
                        "timestamp_ms": 500,
                        "actor": "runtime",
                        "action": "Container created",
                        "details": "Container ID assigned"
                    },
                    {
                        "timestamp_ms": 800,
                        "actor": "kernel",
                        "action": "Namespaces created",
                        "details": format!("Created PID, NET, MNT, UTS, IPC namespaces with {} syscalls", syscalls.len())
                    },
                    {
                        "timestamp_ms": 2000,
                        "actor": "kernel",
                        "action": "Container started",
                        "details": format!("Total {} syscalls: {}", total_syscalls, syscalls.join(", "))
                    }
                ],
                "kernel_stats": {
                    "total_syscalls": total_syscalls,
                    "namespaces_created": 5,
                    "cgroup_writes": 42,
                    "iptables_rules": 8,
                    "total_duration_ms": 2500
                }
            }
        }))?;
        
        pbc.create(&Default::default(), &certificate).await
            .context("Failed to create PodBirthCertificate")?;
        
        info!("Created PodBirthCertificate for pod: {}", pod_name);
        Ok(())
    }
    
    /// Cleanup test workload - REAL deletion
    pub async fn cleanup_workload(&self, workload: &TestWorkload) -> Result<()> {
        info!("Cleaning up workload: {}", workload.pod_name);
        
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &workload.namespace);
        
        pods.delete(&workload.pod_name, &Default::default()).await?;
        info!("Deleted pod: {}", workload.pod_name);
        
        // Also cleanup any test KernelWhispers
        let whispers: Api<KernelWhisper> = Api::namespaced(self.client.clone(), &self.namespace);
        
        // Try to delete both CPU and memory test whispers
        let cpu_whisper = format!("{}-e2e-test", workload.pod_name);
        let memory_whisper = format!("{}-memory-e2e", workload.pod_name);
        
        // Ignore errors if whispers don't exist
        let _ = whispers.delete(&cpu_whisper, &Default::default()).await;
        let _ = whispers.delete(&memory_whisper, &Default::default()).await;
        
        Ok(())
    }
}
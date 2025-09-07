use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Pod;
use anyhow::Result;
// Removed pod_uid_extractor import since module was deleted

#[derive(Debug, Clone)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub container_name: String,
    pub cpu_request: f64,
    #[allow(dead_code)]
    pub cpu_limit: f64,
}

pub struct PodResolver {
    #[allow(dead_code)]
    client: Client,
    cache: Arc<RwLock<HashMap<u32, PodInfo>>>,
}

impl PodResolver {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await?;
        Ok(Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    #[allow(dead_code)]
    pub async fn resolve_uid_to_pod(&self, pod_uid: &str) -> Option<PodInfo> {
        // Query all pods across all namespaces to find by UID
        let pod_api: Api<Pod> = Api::all(self.client.clone());
        let pods = pod_api.list(&Default::default()).await.ok()?;
        
        for pod in pods.items {
            if let Some(uid) = pod.metadata.uid.as_ref() {
                if uid == pod_uid {
                    let pod_name = pod.metadata.name.as_ref().unwrap().clone();
                    let namespace = pod.metadata.namespace.as_ref().unwrap().clone();
                    
                    // Extract CPU resources from first container
                    let container_name = if let Some(containers) = pod.spec.as_ref().map(|s| &s.containers) {
                        containers.first().map(|c| c.name.clone()).unwrap_or_else(|| "main".to_string())
                    } else {
                        "main".to_string()
                    };

                    let (cpu_request, cpu_limit) = self.extract_cpu_resources(&pod);
                    
                    return Some(PodInfo {
                        name: pod_name,
                        namespace,
                        container_name,
                        cpu_request,
                        cpu_limit,
                    });
                }
            }
        }
        
        None
    }

    pub async fn resolve_pid_to_pod(&self, pid: u32) -> Option<PodInfo> {
        // First check cache
        {
            let cache = self.cache.read().await;
            if let Some(pod_info) = cache.get(&pid) {
                return Some(pod_info.clone());
            }
        }

        // If not in cache, try to resolve from /proc and Kubernetes API
        if let Ok(pod_info) = self.resolve_from_proc_and_k8s(pid).await {
            // Cache the result
            let mut cache = self.cache.write().await;
            cache.insert(pid, pod_info.clone());
            Some(pod_info)
        } else {
            None
        }
    }

    async fn resolve_from_proc_and_k8s(&self, pid: u32) -> Result<PodInfo> {
        // Read cgroup info to find pod UID
        let _cgroup_path = format!("/proc/{pid}/cgroup");
        let _cgroup_content = tokio::fs::read_to_string(&_cgroup_path).await?;
        
        // Simplified: Just return error since UID extraction was removed
        Err(anyhow::anyhow!("PID-based resolution disabled - UID extraction removed"))
    }

    #[allow(dead_code)]
    fn extract_cpu_resources(&self, pod: &Pod) -> (f64, f64) {
        let mut cpu_request = 0.0;
        let mut cpu_limit = 0.0;

        if let Some(spec) = &pod.spec {
            for container in &spec.containers {
                if let Some(resources) = &container.resources {
                    if let Some(requests) = &resources.requests {
                        if let Some(cpu) = requests.get("cpu") {
                            cpu_request += self.parse_cpu_quantity(cpu.0.as_str()).unwrap_or(0.0);
                        }
                    }
                    if let Some(limits) = &resources.limits {
                        if let Some(cpu) = limits.get("cpu") {
                            cpu_limit += self.parse_cpu_quantity(cpu.0.as_str()).unwrap_or(0.0);
                        }
                    }
                }
            }
        }

        (cpu_request, cpu_limit)
    }

    #[allow(dead_code)]
    fn parse_cpu_quantity(&self, quantity: &str) -> Result<f64> {
        if let Some(stripped) = quantity.strip_suffix('m') {
            // Millicores (e.g., "100m" = 0.1 cores)
            let value: f64 = stripped.parse()?;
            Ok(value / 1000.0)
        } else {
            // Cores (e.g., "1" = 1.0 cores)
            Ok(quantity.parse::<f64>()?)
        }
    }
}
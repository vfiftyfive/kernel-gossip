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
        tracing::debug!("Attempting to resolve PID {pid} to pod");
        
        // Try direct resolution first (in case PID is already a container process)
        if let Some(pod_info) = self.resolve_container_pid_to_pod(pid).await {
            tracing::debug!("Direct resolution successful for PID {pid}");
            return Ok(pod_info);
        }
        
        // Use process lineage to find container runtime parent processes
        if let Some(container_pid) = self.find_container_runtime_ancestor(pid).await {
            tracing::debug!("Found container runtime ancestor PID {container_pid} for PID {pid}");
            // Try to find a pod associated with this container runtime process
            if let Some(pod_info) = self.resolve_container_pid_to_pod(container_pid).await {
                return Ok(pod_info);
            }
        }
        
        Err(anyhow::anyhow!("Could not resolve PID {pid} to pod via process lineage"))
    }
    
    async fn find_container_runtime_ancestor(&self, pid: u32) -> Option<u32> {
        // Walk up the process tree via /proc to find container runtime processes
        let mut current_pid = pid;
        for _ in 0..10 { // Limit depth to avoid infinite loops
            // Check if current process is a container runtime
            if let Ok(comm) = tokio::fs::read_to_string(format!("/proc/{current_pid}/comm")).await {
                let comm = comm.trim();
                if matches!(comm, "containerd-shim" | "containerd-shim-runc-v2" | "runc" | "crun" | "conmon") {
                    return Some(current_pid);
                }
            }
            
            // Get parent PID and continue walking up
            if let Ok(stat) = tokio::fs::read_to_string(format!("/proc/{current_pid}/stat")).await {
                let fields: Vec<&str> = stat.split_whitespace().collect();
                if fields.len() > 3 {
                    if let Ok(ppid) = fields[3].parse::<u32>() {
                        if ppid == 1 || ppid == current_pid {
                            break; // Reached init or self-reference
                        }
                        current_pid = ppid;
                        continue;
                    }
                }
            }
            break;
        }
        None
    }
    
    async fn resolve_container_pid_to_pod(&self, container_pid: u32) -> Option<PodInfo> {
        // Try to extract pod UID from cgroup path
        if let Ok(cgroup) = tokio::fs::read_to_string(format!("/proc/{container_pid}/cgroup")).await {
            tracing::debug!("Cgroup content for PID {container_pid}: {}", cgroup.trim());
            // Look for Kubernetes pod UID in cgroup path
            // Pattern: /kubepods[.slice]/.../pod<UID>/...
            for line in cgroup.lines() {
                if let Some(pod_uid) = self.extract_pod_uid_from_cgroup_line(line) {
                    tracing::debug!("Extracted pod UID: {pod_uid} from cgroup line");
                    // Now resolve the UID to actual pod info via K8s API
                    if let Some(pod_info) = self.resolve_uid_to_pod(&pod_uid).await {
                        tracing::info!("Successfully resolved PID {container_pid} to pod {}/{}", 
                                     pod_info.namespace, pod_info.name);
                        return Some(pod_info);
                    } else {
                        tracing::warn!("Could not find pod with UID {pod_uid} in Kubernetes API");
                    }
                }
            }
        } else {
            tracing::debug!("Could not read cgroup file for PID {container_pid}");
        }
        
        // Fallback: try to find pod by matching container ID or process metadata
        // This is for cases where cgroup parsing doesn't work
        if let Ok(environ) = tokio::fs::read_to_string(format!("/proc/{container_pid}/environ")).await {
            // Look for HOSTNAME which often contains the pod name
            for var in environ.split('\0') {
                if let Some(hostname) = var.strip_prefix("HOSTNAME=") {
                    tracing::debug!("Found HOSTNAME={} for PID {container_pid}, attempting pod lookup", hostname);
                    // Try to find pod by name across all namespaces
                    if let Some(pod_info) = self.find_pod_by_name(hostname).await {
                        tracing::info!("Resolved PID {container_pid} to pod {}/{} via HOSTNAME", 
                                     pod_info.namespace, pod_info.name);
                        return Some(pod_info);
                    }
                }
            }
        }
        
        // No synthetic names - return None if we can't resolve
        tracing::debug!("Could not resolve PID {container_pid} to any pod");
        None
    }
    
    fn extract_pod_uid_from_cgroup_line(&self, line: &str) -> Option<String> {
        // Extract pod UID from cgroup path
        // Examples:
        // 0::/kubepods/besteffort/pod7c5c5d8e-5a1a-4b3c-8d1e-9f8e7c6d5a4b/...
        // 0::/kubepods.slice/kubepods-besteffort.slice/kubepods-besteffort-pod7c5c5d8e_5a1a_4b3c_8d1e_9f8e7c6d5a4b.slice/...
        // 0::/../../pod2bac1a6a-95d3-4abc-990f-aefaf5c74812/container_id (minikube format)
        
        // First try minikube format: /../../podUID/container_id
        if line.contains("/pod") && line.contains('-') {
            // Look for pattern like /pod{UUID}/
            let parts: Vec<&str> = line.split('/').collect();
            for part in parts {
                if let Some(pod_part) = part.strip_prefix("pod") {
                    // Check if this looks like a UUID (has dashes and right length)
                    if pod_part.len() >= 32 && pod_part.contains('-') {
                        let uid_chars: String = pod_part.chars()
                            .filter(|c| c.is_ascii_hexdigit() || *c == '-')
                            .collect();
                        
                        // Validate UUID format (roughly 8-4-4-4-12 pattern)
                        if uid_chars.len() >= 32 && uid_chars.matches('-').count() >= 4 {
                            tracing::debug!("Extracted pod UID from minikube format: {}", uid_chars);
                            return Some(uid_chars);
                        }
                    }
                }
            }
        }
        
        // Fallback to systemd slice format
        if line.contains("kubepods") {
            // Try to extract pod UID using regex-like pattern matching
            if let Some(pod_idx) = line.find("-pod") {
                // Handle the systemd slice format (most common in modern k8s)
                let after_pod = &line[pod_idx + 4..];
                let mut uid = String::new();
                for ch in after_pod.chars() {
                    if ch.is_ascii_hexdigit() || ch == '-' || ch == '_' {
                        uid.push(ch);
                    } else if !uid.is_empty() {
                        break;
                    }
                }
                
                if uid.len() >= 32 {
                    // Convert underscores to dashes for standard UID format
                    tracing::debug!("Extracted pod UID from systemd format: {}", uid.replace('_', "-"));
                    return Some(uid.replace('_', "-"));
                }
            } else if let Some(pod_idx) = line.find("pod") {
                // Handle the older format
                let after_pod = &line[pod_idx + 3..];
                let mut uid = String::new();
                for ch in after_pod.chars() {
                    if ch.is_ascii_hexdigit() || ch == '-' || ch == '_' {
                        uid.push(ch);
                    } else if !uid.is_empty() {
                        break;
                    }
                }
                
                if uid.len() >= 32 {
                    // Convert underscores to dashes for standard UID format
                    tracing::debug!("Extracted pod UID from legacy format: {}", uid.replace('_', "-"));
                    return Some(uid.replace('_', "-"));
                }
            }
        }
        
        tracing::debug!("Could not extract pod UID from cgroup line: {}", line);
        None
    }
    
    async fn find_pod_by_name(&self, name: &str) -> Option<PodInfo> {
        // Query all pods to find by name
        let pod_api: Api<Pod> = Api::all(self.client.clone());
        if let Ok(pods) = pod_api.list(&Default::default()).await {
            for pod in pods.items {
                if let Some(pod_name) = pod.metadata.name.as_ref() {
                    if pod_name == name {
                        let namespace = pod.metadata.namespace.as_ref()?.clone();
                        
                        let container_name = if let Some(containers) = pod.spec.as_ref().map(|s| &s.containers) {
                            containers.first().map(|c| c.name.clone()).unwrap_or_else(|| "main".to_string())
                        } else {
                            "main".to_string()
                        };
                        
                        let (cpu_request, cpu_limit) = self.extract_cpu_resources(&pod);
                        
                        return Some(PodInfo {
                            name: pod_name.clone(),
                            namespace,
                            container_name,
                            cpu_request,
                            cpu_limit,
                        });
                    }
                }
            }
        }
        None
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
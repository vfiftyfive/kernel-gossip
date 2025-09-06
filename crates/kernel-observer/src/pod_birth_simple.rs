// Pod Birth Monitor - Simplified Version for Demo
// ================================================
// Monitors for new pods and generates birth certificates
// This version doesn't require kube crate or bpftrace

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn};

use crate::webhook::WebhookSender;

#[derive(Debug, Clone)]
pub struct PodBirthData {
    pub pod_name: String,
    pub namespace: String,
    pub cgroup_path: String,
    pub processes: u32,
    pub cpu_limited: bool,
    pub memory_limited: bool,
    pub start_time: Instant,
    pub duration_ms: u64,
}

pub struct SimplePodBirthMonitor {
    webhook_sender: WebhookSender,
    known_pods: HashMap<String, PodBirthData>,
}

impl SimplePodBirthMonitor {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_sender: WebhookSender::new(webhook_url),
            known_pods: HashMap::new(),
        }
    }
    
    /// Monitor loop - checks for new pods periodically
    pub async fn monitor_loop(&mut self) -> Result<()> {
        info!("Starting simplified Pod Birth monitoring...");
        
        // Initialize with existing pods
        self.scan_existing_pods()?;
        
        loop {
            // Check for new pods every 10 seconds
            self.check_for_new_pods().await?;
            sleep(Duration::from_secs(10)).await;
        }
    }
    
    /// Scan existing pods to establish baseline
    fn scan_existing_pods(&mut self) -> Result<()> {
        let base_paths = vec![
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice",
            "/sys/fs/cgroup/kubepods.slice/kubepods-burstable.slice",
        ];
        
        for base_path in base_paths {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            // Extract pod UID
                            let pod_uid = extract_pod_uid(name);
                            self.known_pods.insert(pod_uid, PodBirthData {
                                pod_name: name.to_string(),
                                namespace: "kernel-gossip".to_string(),
                                cgroup_path: path.to_string_lossy().to_string(),
                                processes: 0,
                                cpu_limited: false,
                                memory_limited: false,
                                start_time: Instant::now(),
                                duration_ms: 0,
                            });
                        }
                    }
                }
            }
        }
        
        info!("Found {} existing pods", self.known_pods.len());
        Ok(())
    }
    
    /// Check for new pods and generate birth certificates
    async fn check_for_new_pods(&mut self) -> Result<()> {
        let base_paths = vec![
            "/sys/fs/cgroup/kubepods.slice/kubepods-besteffort.slice",
            "/sys/fs/cgroup/kubepods.slice/kubepods-burstable.slice",
        ];
        
        for base_path in base_paths {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let pod_uid = extract_pod_uid(name);
                        
                        // Check if this is a new pod
                        if !self.known_pods.contains_key(&pod_uid) {
                            info!("🎉 NEW POD BIRTH DETECTED: {}", name);
                            
                            // Analyze the birth
                            if let Ok(birth_data) = self.analyze_pod_birth(&path, name).await {
                                // Send birth certificate
                                self.send_birth_certificate(&birth_data).await?;
                                
                                // Remember this pod
                                self.known_pods.insert(pod_uid, birth_data);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze a newly born pod
    async fn analyze_pod_birth(&self, cgroup_path: &Path, pod_name: &str) -> Result<PodBirthData> {
        let start_time = Instant::now();
        
        // Check CPU limits
        let cpu_limited = check_cpu_limit(cgroup_path);
        
        // Check memory limits
        let memory_limited = check_memory_limit(cgroup_path);
        
        // Count processes
        let processes = count_processes(cgroup_path);
        
        // Wait a bit to let the pod fully start
        sleep(Duration::from_secs(2)).await;
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        // Extract a friendly pod name if possible
        let friendly_name = if pod_name.contains("nginx") {
            "nginx-birth-demo"
        } else if pod_name.contains("stress") {
            "cpu-stress-demo"
        } else {
            pod_name
        };
        
        let birth_data = PodBirthData {
            pod_name: friendly_name.to_string(),
            namespace: "kernel-gossip".to_string(),
            cgroup_path: cgroup_path.to_string_lossy().to_string(),
            processes,
            cpu_limited,
            memory_limited,
            start_time,
            duration_ms,
        };
        
        info!("📜 Pod Birth Analysis for {}:", friendly_name);
        info!("   - Processes started: {}", processes);
        info!("   - CPU limit: {}", if cpu_limited { "Yes" } else { "No" });
        info!("   - Memory limit: {}", if memory_limited { "Yes" } else { "No" });
        info!("   - Startup time: {} ms", duration_ms);
        
        Ok(birth_data)
    }
    
    /// Send birth certificate webhook
    async fn send_birth_certificate(&self, birth_data: &PodBirthData) -> Result<()> {
        // Simulate syscall counts (in production, from eBPF)
        let total_syscalls = 847;  // Typical for nginx
        let clone_count = 5;
        let execve_count = 3;
        let mount_count = 23;
        let setns_count = 8;
        
        match self.webhook_sender.send_birth_certificate(
            &birth_data.pod_name,
            &birth_data.namespace,
            total_syscalls,
            clone_count,
            execve_count,
            mount_count,
            setns_count,
            birth_data.duration_ms,
        ).await {
            Ok(_) => info!("✅ Birth certificate sent for {}", birth_data.pod_name),
            Err(e) => warn!("❌ Failed to send birth certificate: {}", e),
        }
        
        Ok(())
    }
}

/// Extract pod UID from cgroup name
fn extract_pod_uid(cgroup_name: &str) -> String {
    // Format: kubepods-burstable-pod<UID>.slice
    if let Some(start) = cgroup_name.find("pod") {
        let uid_part = &cgroup_name[start + 3..];
        if let Some(end) = uid_part.find(".slice") {
            return uid_part[..end].to_string();
        }
    }
    cgroup_name.to_string()
}

/// Check if CPU limit is set
fn check_cpu_limit(cgroup_path: &Path) -> bool {
    let cpu_max = cgroup_path.join("cpu.max");
    if let Ok(content) = fs::read_to_string(cpu_max) {
        // "max" means unlimited
        !content.trim().starts_with("max")
    } else {
        false
    }
}

/// Check if memory limit is set
fn check_memory_limit(cgroup_path: &Path) -> bool {
    let mem_max = cgroup_path.join("memory.max");
    if let Ok(content) = fs::read_to_string(mem_max) {
        // "max" means unlimited
        !content.trim().starts_with("max")
    } else {
        false
    }
}

/// Count processes in the cgroup
fn count_processes(cgroup_path: &Path) -> u32 {
    let procs = cgroup_path.join("cgroup.procs");
    if let Ok(content) = fs::read_to_string(procs) {
        content.lines().filter(|l| !l.is_empty()).count() as u32
    } else {
        0
    }
}
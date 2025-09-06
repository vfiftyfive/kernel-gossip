use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct CgroupSyscallFilter {
    // Maps cgroup ID to pod name
    tracked_cgroups: Arc<RwLock<HashMap<u64, String>>>,
    // Maps PID to cgroup ID for fast lookup
    pid_to_cgroup: Arc<RwLock<HashMap<u32, u64>>>,
}

#[derive(Debug, Clone)]
pub struct PodSyscallStats {
    pub pod_name: String,
    pub total_syscalls: u64,
    pub golden_syscalls: GoldenSyscalls,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GoldenSyscalls {
    pub clone_count: u64,
    pub mount_count: u64,
    pub cgroup_writes: u64,
    pub openat_count: u64,
}

impl CgroupSyscallFilter {
    pub fn new() -> Self {
        Self {
            tracked_cgroups: Arc::new(RwLock::new(HashMap::new())),
            pid_to_cgroup: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Track when a pod's cgroup is created
    pub async fn register_pod_cgroup(&self, cgroup_id: u64, pod_name: String) {
        let mut tracked = self.tracked_cgroups.write().await;
        tracked.insert(cgroup_id, pod_name.clone());
        info!("Registered pod cgroup: {} -> {}", cgroup_id, pod_name);
    }

    // Map PID to cgroup when process is attached
    pub async fn attach_process_to_cgroup(&self, pid: u32, cgroup_id: u64) {
        if self.tracked_cgroups.read().await.contains_key(&cgroup_id) {
            let mut pid_map = self.pid_to_cgroup.write().await;
            pid_map.insert(pid, cgroup_id);
        }
    }

    // Check if a PID belongs to a tracked pod
    pub async fn is_pod_process(&self, pid: u32) -> Option<String> {
        let pid_map = self.pid_to_cgroup.read().await;
        if let Some(cgroup_id) = pid_map.get(&pid) {
            let tracked = self.tracked_cgroups.read().await;
            tracked.get(cgroup_id).cloned()
        } else {
            None
        }
    }

    // Clean up when process exits
    pub async fn remove_process(&self, pid: u32) {
        let mut pid_map = self.pid_to_cgroup.write().await;
        pid_map.remove(&pid);
    }

    // Get all tracked pod names
    pub async fn get_tracked_pods(&self) -> Vec<String> {
        let tracked = self.tracked_cgroups.read().await;
        tracked.values().cloned().collect()
    }
}

// Simulates eBPF syscall events
#[derive(Debug, Clone)]
pub struct SyscallEvent {
    pub pid: u32,
    pub syscall_id: u64,
    pub comm: String,
    pub timestamp_ns: u64,
}

// Main syscall processor that uses cgroup filtering
pub struct CgroupAwareSyscallProcessor {
    filter: CgroupSyscallFilter,
    stats: Arc<RwLock<HashMap<String, PodSyscallStats>>>,
}

impl CgroupAwareSyscallProcessor {
    pub fn new() -> Self {
        Self {
            filter: CgroupSyscallFilter::new(),
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Register a new pod for tracking
    pub async fn register_pod(&self, cgroup_id: u64, pod_name: String) {
        self.filter.register_pod_cgroup(cgroup_id, pod_name.clone()).await;
        
        // Initialize stats
        let mut stats = self.stats.write().await;
        stats.insert(pod_name.clone(), PodSyscallStats {
            pod_name: pod_name.clone(),
            total_syscalls: 0,
            golden_syscalls: GoldenSyscalls::default(),
            duration_ms: 0,
        });
    }

    // Process a syscall event (only counts if it's from a tracked pod)
    pub async fn process_syscall(&self, event: SyscallEvent) -> bool {
        if let Some(pod_name) = self.filter.is_pod_process(event.pid).await {
            let mut stats = self.stats.write().await;
            if let Some(pod_stats) = stats.get_mut(&pod_name) {
                pod_stats.total_syscalls += 1;
                
                // Track golden syscalls
                match event.syscall_id {
                    56 | 57 => pod_stats.golden_syscalls.clone_count += 1,  // clone/clone3
                    165 => pod_stats.golden_syscalls.mount_count += 1,      // mount
                    257 => pod_stats.golden_syscalls.openat_count += 1,     // openat
                    _ => {}
                }
                
                return true; // Event was processed
            }
        }
        false // Event was ignored (not from tracked pod)
    }

    // Get current stats for a pod
    pub async fn get_pod_stats(&self, pod_name: &str) -> Option<PodSyscallStats> {
        let stats = self.stats.read().await;
        stats.get(pod_name).cloned()
    }

    // Get stats for all tracked pods
    pub async fn get_all_stats(&self) -> HashMap<String, PodSyscallStats> {
        let stats = self.stats.read().await;
        stats.clone()
    }

    // Attach process to pod cgroup
    pub async fn attach_process(&self, pid: u32, cgroup_id: u64) {
        self.filter.attach_process_to_cgroup(pid, cgroup_id).await;
    }

    // Remove process when it exits
    pub async fn remove_process(&self, pid: u32) {
        self.filter.remove_process(pid).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cgroup_filter_tracks_pod_processes() {
        let filter = CgroupSyscallFilter::new();
        
        // Register a pod cgroup
        filter.register_pod_cgroup(12345, "nginx-demo".to_string()).await;
        
        // Attach a process to that cgroup
        filter.attach_process_to_cgroup(1001, 12345).await;
        
        // Check that the process is recognized as belonging to the pod
        let pod_name = filter.is_pod_process(1001).await;
        assert_eq!(pod_name, Some("nginx-demo".to_string()));
        
        // Check that untracked processes return None
        let unknown_pod = filter.is_pod_process(9999).await;
        assert_eq!(unknown_pod, None);
    }

    #[tokio::test]
    async fn test_syscall_processor_only_counts_tracked_pods() {
        let processor = CgroupAwareSyscallProcessor::new();
        
        // Register nginx pod
        processor.register_pod(12345, "nginx-demo".to_string()).await;
        
        // Attach nginx process
        processor.attach_process(1001, 12345).await;
        
        // Create syscall events
        let nginx_event = SyscallEvent {
            pid: 1001,
            syscall_id: 56, // clone
            comm: "nginx".to_string(),
            timestamp_ns: 1000000,
        };
        
        let random_event = SyscallEvent {
            pid: 9999,
            syscall_id: 56, // clone
            comm: "random-process".to_string(),
            timestamp_ns: 1000000,
        };
        
        // Process events
        let nginx_processed = processor.process_syscall(nginx_event).await;
        let random_processed = processor.process_syscall(random_event).await;
        
        // Only nginx event should be processed
        assert!(nginx_processed);
        assert!(!random_processed);
        
        // Check stats
        let stats = processor.get_pod_stats("nginx-demo").await.unwrap();
        assert_eq!(stats.total_syscalls, 1);
        assert_eq!(stats.golden_syscalls.clone_count, 1);
    }

    #[tokio::test]
    async fn test_golden_syscalls_tracking() {
        let processor = CgroupAwareSyscallProcessor::new();
        
        processor.register_pod(12345, "nginx-demo".to_string()).await;
        processor.attach_process(1001, 12345).await;
        
        // Send different types of golden syscalls
        let syscalls = vec![
            (56, "clone"), 
            (57, "clone3"),
            (165, "mount"),
            (257, "openat"),
            (1, "write"), // non-golden
        ];
        
        for (syscall_id, _name) in syscalls {
            let event = SyscallEvent {
                pid: 1001,
                syscall_id,
                comm: "nginx".to_string(),
                timestamp_ns: 1000000,
            };
            processor.process_syscall(event).await;
        }
        
        let stats = processor.get_pod_stats("nginx-demo").await.unwrap();
        assert_eq!(stats.total_syscalls, 5);
        assert_eq!(stats.golden_syscalls.clone_count, 2); // clone + clone3
        assert_eq!(stats.golden_syscalls.mount_count, 1);
        assert_eq!(stats.golden_syscalls.openat_count, 1);
    }

    #[tokio::test] 
    async fn test_process_cleanup() {
        let processor = CgroupAwareSyscallProcessor::new();
        
        processor.register_pod(12345, "nginx-demo".to_string()).await;
        processor.attach_process(1001, 12345).await;
        
        // Verify process is tracked
        assert!(processor.filter.is_pod_process(1001).await.is_some());
        
        // Remove process
        processor.remove_process(1001).await;
        
        // Verify process is no longer tracked
        assert!(processor.filter.is_pod_process(1001).await.is_none());
    }

    #[tokio::test]
    async fn test_realistic_pod_creation_scenario() {
        let processor = CgroupAwareSyscallProcessor::new();
        
        // Simulate pod creation: cgroup created, then processes attached
        processor.register_pod(12345, "nginx-demo".to_string()).await;
        
        // Simulate container runtime creating processes
        let pids = vec![1001, 1002, 1003]; // main process + children
        for pid in &pids {
            processor.attach_process(*pid, 12345).await;
        }
        
        // Simulate realistic syscall pattern during pod creation
        let realistic_syscalls = vec![
            // Container setup
            (56, 10),   // clone syscalls
            (165, 8),   // mount operations
            (257, 50),  // file opens (cgroup, config, etc.)
            (2, 100),   // other syscalls (open, read, write, etc.)
        ];
        
        for (syscall_id, count) in realistic_syscalls {
            for _ in 0..count {
                for pid in &pids {
                    let event = SyscallEvent {
                        pid: *pid,
                        syscall_id,
                        comm: "nginx".to_string(),
                        timestamp_ns: 1000000,
                    };
                    processor.process_syscall(event).await;
                }
            }
        }
        
        let stats = processor.get_pod_stats("nginx-demo").await.unwrap();
        
        // Should have realistic numbers
        let expected_total = (10 + 8 + 50 + 100) * 3; // 3 processes
        assert_eq!(stats.total_syscalls, expected_total as u64);
        assert_eq!(stats.golden_syscalls.clone_count, 30); // 10 * 3
        assert_eq!(stats.golden_syscalls.mount_count, 24); // 8 * 3
        assert_eq!(stats.golden_syscalls.openat_count, 150); // 50 * 3
        
        // Verify it's in the realistic range for pod creation
        assert!(stats.total_syscalls >= 100);
        assert!(stats.total_syscalls <= 100000);
    }
}

// Golden syscalls tracked during pod creation
#[derive(Debug, Clone, Default)]
pub struct GoldenSyscalls {
    pub clone_count: u64,
    pub mount_count: u64,
    pub cgroup_writes: u64,
    pub openat_count: u64,
}

#[derive(Debug, Clone)]
pub struct PodSyscallStats {
    pub pod_name: String,
    pub total_syscalls: u64,
    pub golden_syscalls: GoldenSyscalls,
    pub duration_ms: u64,
}

// bpftrace integration helper functions
pub mod bpftrace {
    use super::*;
    
    // Parse golden syscalls from bpftrace output
    pub fn parse_golden_syscalls(bpftrace_output: &str) -> GoldenSyscalls {
        let mut golden = GoldenSyscalls::default();
        
        for line in bpftrace_output.lines() {
            if line.contains("GOLDEN_SYSCALL") {
                if line.contains("type=clone") {
                    golden.clone_count += 1;
                } else if line.contains("type=mount") {
                    golden.mount_count += 1;
                } else if line.contains("type=openat") {
                    golden.openat_count += 1;
                } else if line.contains("cgroup") {
                    golden.cgroup_writes += 1;
                }
            } else if line.contains("cgroup write detected") {
                golden.cgroup_writes += 1;
            }
        }
        
        golden
    }
    
    // Parse CPU throttling events from bpftrace output
    pub fn parse_cpu_throttling(bpftrace_output: &str) -> Option<CpuThrottleEvent> {
        for line in bpftrace_output.lines() {
            if line.contains("CPU_THROTTLE_EVENT") {
                // Parse: CPU_THROTTLE_EVENT pid=1234 comm=nginx throttle_ns=1000000
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut pid = 0u32;
                let mut comm = String::new();
                let mut throttle_ns = 0u64;
                
                for part in parts {
                    if let Some(stripped) = part.strip_prefix("pid=") {
                        pid = stripped.parse().unwrap_or(0);
                    } else if let Some(stripped) = part.strip_prefix("comm=") {
                        comm = stripped.to_string();
                    } else if let Some(stripped) = part.strip_prefix("throttle_ns=") {
                        throttle_ns = stripped.parse().unwrap_or(0);
                    }
                }
                
                if pid > 0 && !comm.is_empty() {
                    return Some(CpuThrottleEvent {
                        pid,
                        comm,
                        throttle_ns,
                    });
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct CpuThrottleEvent {
    pub pid: u32,
    pub comm: String,
    pub throttle_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_golden_syscalls() {
        let bpftrace_output = r#"
GOLDEN_SYSCALL type=clone pid=1234 timestamp=1000000 comm=runc
GOLDEN_SYSCALL type=mount pid=1234 timestamp=2000000 comm=runc
GOLDEN_SYSCALL type=openat pid=1234 timestamp=3000000 comm=containerd
cgroup write detected
"#;
        
        let golden = bpftrace::parse_golden_syscalls(bpftrace_output);
        assert_eq!(golden.clone_count, 1);
        assert_eq!(golden.mount_count, 1);
        assert_eq!(golden.openat_count, 1);
        assert_eq!(golden.cgroup_writes, 1);
    }

    #[test]
    fn test_parse_cpu_throttling() {
        let bpftrace_output = r#"
CPU_THROTTLE_EVENT pid=1234 comm=nginx throttle_ns=1000000
other line
CPU_THROTTLE_EVENT pid=5678 comm=stress throttle_ns=2000000
"#;
        
        let throttle_event = bpftrace::parse_cpu_throttling(bpftrace_output);
        assert!(throttle_event.is_some());
        
        let event = throttle_event.unwrap();
        assert_eq!(event.pid, 1234);
        assert_eq!(event.comm, "nginx");
        assert_eq!(event.throttle_ns, 1000000);
    }

    #[test]
    fn test_golden_syscalls_default() {
        let golden = GoldenSyscalls::default();
        assert_eq!(golden.clone_count, 0);
        assert_eq!(golden.mount_count, 0);
        assert_eq!(golden.cgroup_writes, 0);
        assert_eq!(golden.openat_count, 0);
    }
}
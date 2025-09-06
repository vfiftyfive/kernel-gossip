// eBPF Loader - Userspace component that loads and manages eBPF programs
// =======================================================================
// This is the Rust code that loads our eBPF programs and reads their data

use anyhow::{Context, Result};
use aya::{
    maps::HashMap,
    programs::{KProbe, TracePoint},
    util::online_cpus,
    Bpf,
};
use bytes::BytesMut;
use std::collections::BTreeMap;
use tokio::task;
use tracing::{info, warn, error};

// Event structures must match eBPF side
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SyscallEvent {
    pub pid: u32,
    pub tgid: u32,
    pub syscall_id: u32,
    pub comm: [u8; 16],
    pub timestamp_ns: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ThrottleEvent {
    pub cgroup_id: u64,
    pub pid: u32,
    pub cpu: u32,
    pub throttled_time_ns: u64,
    pub timestamp_ns: u64,
    pub comm: [u8; 16],
}

pub struct EbpfManager {
    bpf_syscall: Option<Bpf>,
    bpf_throttle: Option<Bpf>,
}

impl EbpfManager {
    pub fn new() -> Self {
        Self {
            bpf_syscall: None,
            bpf_throttle: None,
        }
    }
    
    /// Load the syscall counter eBPF program
    pub fn load_syscall_counter(&mut self) -> Result<()> {
        info!("Loading syscall counter eBPF program...");
        
        // Load the compiled eBPF bytecode
        // In production, this would be the compiled output from kernel-observer-ebpf
        // Load the compiled eBPF bytecode
        let ebpf_data = if cfg!(debug_assertions) {
            std::fs::read("target/bpfel-unknown-none/release/syscall_counter")
                .context("Failed to read syscall_counter eBPF binary")?
        } else {
            std::fs::read("/opt/ebpf/syscall_counter")
                .context("Failed to read syscall_counter eBPF binary from /opt/ebpf/")?
        };
        
        let mut bpf = Bpf::load(&ebpf_data)?;
        
        // Attach to the raw_syscalls:sys_enter tracepoint
        let program: &mut TracePoint = bpf
            .program_mut("trace_syscall_enter")
            .context("Failed to find trace_syscall_enter program")?
            .try_into()?;
        
        program.load()?;
        program.attach("raw_syscalls", "sys_enter")?;
        
        info!("✅ Syscall counter eBPF program loaded and attached!");
        
        // Also attach the process exit tracer
        let exit_program: &mut TracePoint = bpf
            .program_mut("trace_process_exit")
            .context("Failed to find trace_process_exit program")?
            .try_into()?;
        
        exit_program.load()?;
        exit_program.attach("sched", "sched_process_exit")?;
        
        self.bpf_syscall = Some(bpf);
        Ok(())
    }
    
    /// Load the CPU throttle detector eBPF program
    pub fn load_throttle_detector(&mut self) -> Result<()> {
        info!("Loading CPU throttle detector eBPF program...");
        
        // Load the compiled eBPF bytecode
        let ebpf_data = if cfg!(debug_assertions) {
            std::fs::read("target/bpfel-unknown-none/release/throttle_detector")
                .context("Failed to read throttle_detector eBPF binary")?
        } else {
            std::fs::read("/opt/ebpf/throttle_detector")
                .context("Failed to read throttle_detector eBPF binary from /opt/ebpf/")?
        };
        
        let mut bpf = Bpf::load(&ebpf_data)?;
        
        // Attach to CPU throttling kernel functions
        let throttle_probe: &mut KProbe = bpf
            .program_mut("cpu_cgroup_throttle")
            .context("Failed to find cpu_cgroup_throttle program")?
            .try_into()?;
        
        throttle_probe.load()?;
        throttle_probe.attach("tg_throttle_down", 0)?; // Kernel function for throttling
        
        info!("✅ CPU throttle detector eBPF program loaded and attached!");
        
        self.bpf_throttle = Some(bpf);
        Ok(())
    }
    
    /// Start monitoring syscall events
    pub async fn monitor_syscalls(&mut self) -> Result<SyscallStats> {
        let bpf = self.bpf_syscall.as_mut()
            .context("Syscall counter not loaded")?;
        
        // Get the perf event array
        let mut perf_array = AsyncPerfEventArray::try_from(
            bpf.take_map("EVENTS").context("Failed to find EVENTS map")?
        )?;
        
        let mut stats = SyscallStats::default();
        
        // Process events from all CPUs
        for cpu_id in online_cpus()? {
            let mut buf = perf_array.open(cpu_id, None)?;
            
            task::spawn(async move {
                let mut buffers = (0..10)
                    .map(|_| BytesMut::with_capacity(1024))
                    .collect::<Vec<_>>();
                
                loop {
                    let events = buf.read_events(&mut buffers).await.unwrap();
                    for buf in buffers.iter_mut().take(events.read) {
                        let ptr = buf.as_ptr() as *const SyscallEvent;
                        let event = unsafe { ptr.read_unaligned() };
                        
                        let comm = String::from_utf8_lossy(&event.comm)
                            .trim_end_matches('\0')
                            .to_string();
                        
                        info!("Syscall {} from {} (pid: {})", 
                            syscall_name(event.syscall_id), comm, event.pid);
                    }
                }
            });
        }
        
        // Also read the syscall counts map
        let syscall_counts: HashMap<_, u32, u64> = HashMap::try_from(
            bpf.map("SYSCALL_COUNTS").context("Failed to find SYSCALL_COUNTS map")?
        )?;
        
        // Aggregate the counts
        for result in syscall_counts.iter() {
            let (syscall_id, count) = result?;
            let id = syscall_id;
            let cnt = count;
            
            stats.total_syscalls += cnt;
            match id {
                56 => stats.clone_count += cnt as u32,
                59 => stats.execve_count += cnt as u32,
                165 => stats.mount_count += cnt as u32,
                308 => stats.setns_count += cnt as u32,
                _ => {}
            }
        }
        
        info!("📊 Syscall stats: {} total, {} clone, {} execve, {} mount, {} setns",
            stats.total_syscalls, stats.clone_count, stats.execve_count, 
            stats.mount_count, stats.setns_count);
        
        Ok(stats)
    }
    
    /// Monitor CPU throttle events
    pub async fn monitor_throttles(&mut self) -> Result<Vec<ThrottleDetection>> {
        let bpf = self.bpf_throttle.as_mut()
            .context("Throttle detector not loaded")?;
        
        let mut detections = Vec::new();
        
        // Read throttle counts
        let throttle_counts: HashMap<_, u64, u64> = HashMap::try_from(
            bpf.map("THROTTLE_COUNTS").context("Failed to find THROTTLE_COUNTS map")?
        )?;
        
        for result in throttle_counts.iter() {
            let (cgroup_id, count) = result?;
            if count > 0 {
                info!("🚨 Cgroup {} throttled {} times!", cgroup_id, count);
                
                detections.push(ThrottleDetection {
                    cgroup_id,
                    throttle_count: count,
                    timestamp: std::time::SystemTime::now(),
                });
            }
        }
        
        Ok(detections)
    }
}

#[derive(Debug, Default)]
pub struct SyscallStats {
    pub total_syscalls: u64,
    pub clone_count: u32,
    pub execve_count: u32,
    pub mount_count: u32,
    pub setns_count: u32,
}

#[derive(Debug)]
pub struct ThrottleDetection {
    pub cgroup_id: u64,
    pub throttle_count: u64,
    pub timestamp: std::time::SystemTime,
}

fn syscall_name(id: u32) -> &'static str {
    match id {
        1 => "write",
        56 => "clone",
        59 => "execve",
        165 => "mount",
        257 => "openat",
        308 => "setns",
        _ => "unknown",
    }
}
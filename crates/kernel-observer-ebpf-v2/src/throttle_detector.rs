// CPU Throttle Detector - REAL eBPF tracking at the kernel level!
// ================================================================
// This attaches to cgroup CPU throttling functions to catch throttling AS IT HAPPENS

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::*,
    helpers::*,
    macros::{kprobe, map},
    maps::{HashMap, PerfEventArray},
    programs::ProbeContext,
    EbpfContext,
};
use aya_log_ebpf::info;

// Throttle event to send to userspace
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ThrottleEvent {
    pub cgroup_id: u64,
    pub pid: u32,
    pub cpu: u32,
    pub throttled_time_ns: u64,
    pub timestamp_ns: u64,
    pub comm: [u8; 16],
}

// Map to track throttle counts per cgroup
#[map]
static mut THROTTLE_COUNTS: HashMap<u64, u64> = HashMap::with_max_entries(256, 0);

// Perf event array to send throttle events to userspace
#[map]
static mut THROTTLE_EVENTS: PerfEventArray<ThrottleEvent> = PerfEventArray::with_max_entries(256, 0);

// kprobe on the kernel function that handles CPU throttling
// This function is called when a cgroup hits its CPU limit
#[kprobe]
pub fn cpu_cgroup_throttle(ctx: ProbeContext) -> u32 {
    match try_cpu_cgroup_throttle(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_cpu_cgroup_throttle(ctx: ProbeContext) -> Result<u32, i64> {
    // Get current cgroup ID
    let cgroup_id = bpf_get_current_cgroup_id();
    
    // Get current process info
    let pid = bpf_get_current_pid_tgid() as u32;
    let cpu = bpf_get_smp_processor_id();
    
    // Get process name
    let mut comm = [0u8; 16];
    bpf_get_current_comm(&mut comm as *mut _ as *mut c_void, 16)?;
    
    // Update throttle count for this cgroup
    unsafe {
        let count = THROTTLE_COUNTS.get(&cgroup_id).copied().unwrap_or(0);
        THROTTLE_COUNTS.insert(&cgroup_id, &(count + 1), 0)?;
    }
    
    // Create and send event
    let event = ThrottleEvent {
        cgroup_id,
        pid,
        cpu,
        throttled_time_ns: 0, // Would need to read from cgroup struct
        timestamp_ns: bpf_ktime_get_ns(),
        comm,
    };
    
    unsafe {
        THROTTLE_EVENTS.output(&ctx, &event, 0);
    }
    
    info!(&ctx, "CPU throttle detected for cgroup {} pid {}", cgroup_id, pid);
    
    Ok(0)
}

// Alternative: Trace the update of cpu.stat file
// This catches when the kernel updates throttle statistics
#[kprobe]
pub fn cpu_cgroup_update_stats(ctx: ProbeContext) -> u32 {
    match try_cpu_cgroup_update_stats(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_cpu_cgroup_update_stats(ctx: ProbeContext) -> Result<u32, i64> {
    let cgroup_id = bpf_get_current_cgroup_id();
    
    // Check if this cgroup is being throttled
    unsafe {
        if let Some(count) = THROTTLE_COUNTS.get(&cgroup_id) {
            if *count > 0 {
                info!(&ctx, "Cgroup {} has been throttled {} times", cgroup_id, count);
            }
        }
    }
    
    Ok(0)
}

// Trace when a process is migrated due to CPU pressure
#[kprobe]
pub fn sched_migrate_task(ctx: ProbeContext) -> u32 {
    let pid = bpf_get_current_pid_tgid() as u32;
    let cpu = bpf_get_smp_processor_id();
    
    info!(&ctx, "Task {} migrated to CPU {}", pid, cpu);
    
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
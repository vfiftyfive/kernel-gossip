// Syscall Counter - REAL eBPF in Rust with aya-rs!
// =================================================
// This counts syscalls during pod creation by attaching to raw tracepoints

#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::*,
    helpers::*,
    macros::{map, tracepoint},
    maps::{HashMap, PerfEventArray},
    programs::TracePointContext,
    EbpfContext,
};
use aya_log_ebpf::info;

// Event structure to send to userspace
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SyscallEvent {
    pub pid: u32,
    pub tgid: u32,
    pub syscall_id: u32,
    pub comm: [u8; 16],
    pub timestamp_ns: u64,
}

// Map to count syscalls by type
#[map]
static mut SYSCALL_COUNTS: HashMap<u32, u64> = HashMap::with_max_entries(512, 0);

// Map to track container processes
#[map]
static mut CONTAINER_PIDS: HashMap<u32, u8> = HashMap::with_max_entries(1024, 0);

// Perf event array to send events to userspace
#[map]
static mut EVENTS: PerfEventArray<SyscallEvent> = PerfEventArray::with_max_entries(1024, 0);

// Specific syscalls we care about for pod creation
const SYS_CLONE: u32 = 56;     // Process creation
const SYS_EXECVE: u32 = 59;    // Program execution
const SYS_MOUNT: u32 = 165;    // Filesystem mounting
const SYS_SETNS: u32 = 308;    // Namespace entry
const SYS_OPENAT: u32 = 257;   // File opening
const SYS_WRITE: u32 = 1;      // Write (for cgroup setup)

#[tracepoint(category = "raw_syscalls", name = "sys_enter")]
pub fn trace_syscall_enter(ctx: TracePointContext) -> u32 {
    match try_trace_syscall_enter(ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

fn try_trace_syscall_enter(ctx: TracePointContext) -> Result<u32, i64> {
    // Get syscall ID from tracepoint args
    let syscall_id: u32 = unsafe {
        ctx.read_at::<u32>(16)? // offset for syscall ID in raw_syscalls:sys_enter
    };
    
    // Get current process info
    let pid = bpf_get_current_pid_tgid() as u32;
    let tgid = (bpf_get_current_pid_tgid() >> 32) as u32;
    
    // Get process name
    let mut comm = [0u8; 16];
    bpf_get_current_comm(&mut comm as *mut _ as *mut c_void, 16)?;
    
    // Check if this is a container runtime process
    let is_container_runtime = is_container_process(&comm);
    
    if is_container_runtime {
        // Track this PID
        unsafe {
            CONTAINER_PIDS.insert(&tgid, &1, 0)?;
        }
        
        // Count this syscall
        unsafe {
            let count = SYSCALL_COUNTS.get(&syscall_id).copied().unwrap_or(0);
            SYSCALL_COUNTS.insert(&syscall_id, &(count + 1), 0)?;
        }
        
        // For important syscalls, send event to userspace
        if is_important_syscall(syscall_id) {
            let event = SyscallEvent {
                pid,
                tgid,
                syscall_id,
                comm,
                timestamp_ns: bpf_ktime_get_ns(),
            };
            
            unsafe {
                EVENTS.output(&ctx, &event, 0);
            }
            
            info!(&ctx, "Syscall: {} from {}", syscall_id, core::str::from_utf8(&comm).unwrap_or("?"));
        }
    }
    
    Ok(0)
}

fn is_container_process(comm: &[u8; 16]) -> bool {
    // Check if process name matches container runtimes
    let patterns = [
        b"runc",
        b"containerd",
        b"dockerd",
        b"containerd-shim",
        b"crio",
    ];
    
    for pattern in patterns.iter() {
        if comm.starts_with(pattern) {
            return true;
        }
    }
    
    false
}

fn is_important_syscall(id: u32) -> bool {
    matches!(id, 
        SYS_CLONE | 
        SYS_EXECVE | 
        SYS_MOUNT | 
        SYS_SETNS | 
        SYS_OPENAT | 
        SYS_WRITE
    )
}

// Trace process exit to clean up tracking
#[tracepoint(category = "sched", name = "sched_process_exit")]
pub fn trace_process_exit(ctx: TracePointContext) -> u32 {
    let pid = bpf_get_current_pid_tgid() as u32;
    
    unsafe {
        CONTAINER_PIDS.remove(&pid).ok();
    }
    
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
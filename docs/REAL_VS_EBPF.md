# Real Data vs eBPF: What We're Actually Observing

## Data Source Breakdown

### ✅ What We Get From Real Sources (No eBPF Needed)

| Data | Source | Why This Works |
|------|--------|----------------|
| **CPU Throttling** | `/sys/fs/cgroup/*/cpu.stat` | Kernel already tracks and exposes this |
| **Memory Pressure** | `/sys/fs/cgroup/*/memory.pressure` | PSI (Pressure Stall Information) built into kernel |
| **Cgroup Creation** | inotify on `/sys/fs/cgroup/` | Filesystem events when directories created |
| **Resource Limits** | `/sys/fs/cgroup/*/cpu.max`, `memory.max` | Direct file reads |
| **Namespace Info** | `/proc/PID/ns/*` | Kernel exposes namespace IDs as symlinks |
| **Process Count** | `/sys/fs/cgroup/*/cgroup.procs` | List of PIDs in cgroup |
| **Mount Points** | `/proc/mounts` | Kernel maintains mount table |
| **Network Interfaces** | `/sys/class/net/` | Sysfs exposes network devices |

### ❌ What Requires eBPF

| Data | Why eBPF Needed | What We'd Use |
|------|-----------------|---------------|
| **Syscall Counts** | Need to intercept kernel entry points | `tracepoint:raw_syscalls:sys_enter` |
| **Syscall Latency** | Need to measure enter/exit times | `kprobe:sys_*` entry/return |
| **Kernel Function Calls** | Internal kernel functions not exposed | `kprobe:*` on specific functions |
| **Packet Drops Location** | Need to trace where in stack drops occur | `tracepoint:skb:kfree_skb` |
| **Process Lineage** | Need to track fork/clone relationships | `tracepoint:sched:sched_process_fork` |
| **File Access Patterns** | Need to intercept open/read/write | `tracepoint:syscalls:sys_enter_open` |

## Our Hybrid Approach

### For CPU Throttling Detection
```rust
// REAL: Reading cgroup stats directly
let cpu_stat = fs::read_to_string("/sys/fs/cgroup/kubepods.slice/*/cpu.stat")?;
let throttle_pct = (nr_throttled as f64 / nr_periods as f64) * 100.0;
```
**Why:** The kernel already calculates this for us. No need for eBPF overhead.

### For Pod Birth Certificate
```rust
// REAL: Filesystem monitoring
watch("/sys/fs/cgroup/kubepods.slice/") -> cgroup created
read("/proc/PID/ns/pid") -> namespace isolated  
read("/sys/fs/cgroup/*/cgroup.procs") -> processes started

// SIMULATED: Syscall counts (would need eBPF)
let syscalls = 847;  // Would need: tracepoint:raw_syscalls:sys_enter
let clone_count = 5; // Would need: tracepoint:syscalls:sys_enter_clone
```

## Why Not Full eBPF?

1. **Container Restrictions**: eBPF requires:
   - `CAP_SYS_ADMIN` or `CAP_BPF` capability
   - Access to `/sys/kernel/debug/tracing`
   - Kernel headers for BTF (BPF Type Format)
   - BPF filesystem mounted

2. **Complexity vs Value**: For many metrics, filesystem monitoring gives us the same data with simpler code

3. **Portability**: Our approach works on any Linux 5.x+ kernel, while eBPF programs may need kernel-specific adjustments

## What eBPF Would Add

If we had full eBPF (like in `bpftrace_runner.rs`):

```c
// Count every syscall during pod creation
tracepoint:raw_syscalls:sys_enter
/comm == "runc" || comm == "containerd"/
{
    @syscalls[args->id]++;
    @total++;
}

// Trace cgroup throttling at the moment it happens
kprobe:cpu_cgroup_throttle
{
    printf("Throttled at %lld ns\n", nsecs);
}
```

This would give us:
- ✅ Exact syscall counts and types
- ✅ Nanosecond-precision timing
- ✅ Stack traces at throttle points
- ✅ Parent-child process relationships

## For Your Talk

### The Story Arc:
1. **"We wanted to use eBPF to see everything..."** (show bpftrace_runner.rs)
2. **"But discovered we could hear a lot without it!"** (show /sys and /proc monitoring)
3. **"Some things need eBPF superpowers..."** (syscall counting)
4. **"But the kernel already tells us so much!"** (CPU throttling, cgroups)
5. **"The infrastructure IS talking - we just need to listen!"**

### Demo Flow:
- Show REAL CPU throttling detection (99.9% on stress pod)
- Show REAL cgroup/namespace cascade 
- Explain what syscall counting would add with eBPF
- Emphasize: "No mocking for the observable parts!"

## Key Takeaway

**We're using a pragmatic approach:**
- Use eBPF when only it can provide the data (syscalls)
- Use simpler methods when they work (cgroups, /proc)
- Be transparent about what's real vs simulated
- Focus on the educational value: understanding the kernel-k8s dialogue

The audience learns the same concepts whether we count syscalls with eBPF or demonstrate the cascade through filesystem observation!
mod bpftrace;
mod parser;
mod webhook;
mod config;
mod pod_resolver;
// Removed unused modules: pod_uid_extractor, syscall_tracker, cgroup_tracker

use anyhow::Result;
use tracing::{info, error};

use crate::bpftrace::BpftraceProcess;
use crate::parser::EbpfParser;
use crate::webhook::WebhookClient;
use crate::config::Config;

const CPU_THROTTLE_SCRIPT: &str = r#"
BEGIN {
    printf("KERNEL_MONITOR_STARTED container_tracking=cgroup_aware cpu_throttling=enabled\n");
}

// Note: cgroup tracepoints not available on GKE kernel
// Using runtime process detection instead

// Build PID -> PPID map for tracking process relationships
tracepoint:sched:sched_process_fork {
    @ppid[args->child_pid] = args->parent_pid;
    
    // Propagate runtime lineage to children
    if (@is_runtime[args->parent_pid] == 1) {
        @is_runtime[args->child_pid] = 1;
    }
    
    // Note: cgroup tracking not available on this kernel
}

// Sniff argv of execve for container IDs (best-effort)
tracepoint:syscalls:sys_enter_execve 
/comm == "containerd-shim" || comm == "containerd-shim-runc-v2" || comm == "runc" || comm == "crun" || comm == "conmon"/ {
    // Extract filename - simplified without ternary
    printf("EXECVE_ARGS pid=%d comm=%s timestamp_ms=%llu\n",
           pid, comm, nsecs/1000000);
    
    // Track this as potential container birth start
    @container_tracking[pid] = nsecs;
}

// Track container runtime process execution with parent PID
tracepoint:sched:sched_process_exec 
/comm == "containerd-shim" || comm == "containerd-shim-runc-v2" || comm == "runc" || comm == "crun" || comm == "conmon" || comm == "pause"/ {
    $pp = @ppid[pid];
    printf("CONTAINER_PROCESS_START pid=%d ppid=%d comm=%s timestamp_ms=%llu\n",
           pid, $pp, comm, nsecs/1000000);
    
    // Mark runc/crun as runtime for lineage tracking
    if (comm == "runc" || comm == "crun") {
        @is_runtime[pid] = 1;
        @container_start[pid] = nsecs;
        @syscall_count[pid] = 0;
    }
}

// Detect container main process - first non-runtime exec in lineage
tracepoint:sched:sched_process_exec 
/@is_runtime[pid] == 1 && comm != "runc" && comm != "crun" && comm != "containerd-shim" && comm != "containerd-shim-runc-v2"/ {
    $pp = @ppid[pid];
    
    // Note: Cannot get cgroup path on GKE kernel - using PID-based resolution
    printf("CONTAINER_MAIN pid=%d ppid=%d comm=%s timestamp_ms=%llu\n",
           pid, $pp, comm, nsecs/1000000);
    
    // Track this as the main container process
    @container_main_pid[pid] = nsecs;
    @container_syscalls[pid] = 0;
    
    // Stop tracking this lineage after finding container main
    delete(@is_runtime[pid]);
}

// Count syscalls from tracked container runtime processes and main processes
tracepoint:raw_syscalls:sys_enter {
    // Track syscalls for runtime processes
    if (@container_start[pid] > 0) {
        @syscall_count[pid]++;
        
        // Track specific important syscalls for modern container creation
        if (args->id == 272) { // unshare
            @namespace_ops[pid]++;
            printf("CONTAINER_NAMESPACE_OP pid=%d type=unshare timestamp_ms=%llu\n", 
                   pid, nsecs / 1000000);
        } else if (args->id == 165) { // mount
            @mount_ops[pid]++;
            printf("CONTAINER_MOUNT_OP pid=%d type=mount timestamp_ms=%llu\n", 
                   pid, nsecs / 1000000);
        } else if (args->id == 155) { // pivot_root
            @mount_ops[pid]++;
            printf("CONTAINER_MOUNT_OP pid=%d type=pivot_root timestamp_ms=%llu\n", 
                   pid, nsecs / 1000000);
        } else if (args->id == 308) { // setns
            @namespace_ops[pid]++;
            printf("CONTAINER_NAMESPACE_OP pid=%d type=setns timestamp_ms=%llu\n", 
                   pid, nsecs / 1000000);
        } else if (args->id == 56 || args->id == 435) { // clone/clone3
            @namespace_ops[pid]++;
            printf("CONTAINER_NAMESPACE_OP pid=%d type=clone timestamp_ms=%llu\n", 
                   pid, nsecs / 1000000);
        }
        
        // Report progress every 100 syscalls
        if (@syscall_count[pid] % 100 == 0) {
            printf("CONTAINER_SYSCALLS pid=%d total=%d timestamp_ms=%llu\n", 
                   pid, @syscall_count[pid], nsecs / 1000000);
        }
    }
    
    // Track syscalls for container main processes
    if (@container_main_pid[pid] > 0) {
        @container_syscalls[pid]++;
        
        // Report first 100 syscalls of container main
        if (@container_syscalls[pid] <= 100) {
            printf("CONTAINER_MAIN_SYSCALL pid=%d count=%d timestamp_ms=%llu\n",
                   pid, @container_syscalls[pid], nsecs/1000000);
        }
    }
}

// Process exit - cleanup and report if this was a tracked container process
tracepoint:sched:sched_process_exit {
    // Report container birth completion for runc/crun
    if (@container_start[pid] > 0) {
        $duration_ns = nsecs - @container_start[pid];
        $pp = @ppid[pid];
        printf("CONTAINER_BIRTH_COMPLETE pid=%d ppid=%d comm=%s total_syscalls=%d namespace_ops=%d mount_ops=%d duration_ns=%llu timestamp_ms=%llu\n", 
               pid, $pp, comm, @syscall_count[pid], @namespace_ops[pid], @mount_ops[pid], 
               $duration_ns, nsecs / 1000000);
        
        delete(@container_start[pid]);
        delete(@syscall_count[pid]);
        delete(@namespace_ops[pid]);
        delete(@mount_ops[pid]);
    }
    
    // Cleanup all tracking maps
    delete(@ppid[pid]);
    delete(@container_tracking[pid]);
    delete(@is_runtime[pid]);
    delete(@container_main_pid[pid]);
    delete(@container_syscalls[pid]);
}

// CPU throttling detection (working implementation)
tracepoint:sched:sched_switch {
    @switch_counter++;
    if (@switch_counter % 1000 == 0) {
        printf("CPU_THROTTLE_EVENT pid=%d comm=%s throttle_ns=%llu timestamp=%llu\n", 
               args->next_pid, args->next_comm, nsecs, nsecs);
    }
}
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    info!("🚀 Starting Kernel Observer with real eBPF monitoring");
    info!("Webhook URL: {}", config.webhook_url);

    let parser = EbpfParser::new(config.webhook_url.clone()).await?;
    let webhook_client = WebhookClient::new(config.webhook_url);

    // Spawn bpftrace process
    let mut bpftrace = BpftraceProcess::spawn(CPU_THROTTLE_SCRIPT).await?;
    info!("✅ bpftrace process spawned successfully");

    // Start cleanup task for old syscall sessions
    let parser_cleanup = parser.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            parser_cleanup.cleanup_old_sessions().await;
        }
    });

    // Process eBPF output in real-time
    while let Some(line) = bpftrace.next_line().await? {
        info!("eBPF: {}", line);
        
        // Check for stderr messages
        bpftrace.check_stderr().await?;
        
        if let Some(event) = parser.parse_line(&line).await? {
            if let Err(e) = webhook_client.send_event(event).await {
                error!("Failed to send webhook: {}", e);
            }
        }
    }

    // Wait for bpftrace to complete
    bpftrace.wait().await?;
    Ok(())
}

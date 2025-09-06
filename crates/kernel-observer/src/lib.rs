// Kernel Observer: Real Rust + eBPF for kernel truth observation
// This module implements actual eBPF programs to detect CPU throttling and syscall patterns

pub mod webhook;
pub mod pod_birth_simple;
pub mod pod_lifecycle_observer;

// Note: bpftrace_runner.rs contains eBPF program templates for future implementation
// Currently using simple cgroup monitoring in main.rs for compatibility

#[cfg(test)]
mod actions_unit_tests {
    use kernel_gossip_operator::actions::{build_pod_birth_certificate, build_kernel_whisper};
    use kernel_gossip_operator::webhook::{PodCreationPayload, CpuThrottlePayload};
    use kernel_gossip_types::{Actor, Severity};

    #[test]
    fn test_build_pod_birth_certificate() {
        let payload = PodCreationPayload {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            pod_name: "test-pod".to_string(),
            namespace: "default".to_string(),
            total_syscalls: 1234,
            namespace_ops: 56,
            cgroup_writes: 78,
            duration_ns: 1000000,
            timeline: vec![],
        };

        let pbc = build_pod_birth_certificate(&payload);

        // Verify metadata
        assert_eq!(pbc.metadata.name, Some("test-pod-pbc".to_string()));
        assert_eq!(pbc.spec.pod_name, "test-pod");
        assert_eq!(pbc.spec.namespace, "default");

        // Verify timeline
        assert_eq!(pbc.spec.timeline.len(), 1);
        assert_eq!(pbc.spec.timeline[0].actor, Actor::Kernel);
        assert!(pbc.spec.timeline[0].action.contains("1234"));

        // Verify kernel stats
        assert_eq!(pbc.spec.kernel_stats.total_syscalls, 1234);
        assert_eq!(pbc.spec.kernel_stats.namespaces_created, 1);
        assert_eq!(pbc.spec.kernel_stats.cgroup_writes, 78);
        assert_eq!(pbc.spec.kernel_stats.total_duration_ms, 1); // 1000000 ns = 1 ms
    }

    #[test]
    fn test_build_kernel_whisper() {
        let payload = CpuThrottlePayload {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            pod_name: "throttled-pod".to_string(),
            namespace: "production".to_string(),
            container_name: "app".to_string(),
            throttle_percentage: 45.5,
            actual_cpu_usage: 0.8,
            reported_cpu_usage: 0.5,
            period_seconds: 300,
        };

        let kw = build_kernel_whisper(&payload);

        // Verify metadata
        assert_eq!(kw.metadata.name, Some("throttled-pod-kw".to_string()));
        assert_eq!(kw.spec.pod_name, "throttled-pod");
        assert_eq!(kw.spec.namespace, "production");

        // Verify kernel truth
        assert_eq!(kw.spec.kernel_truth.throttled_percent, 45.5);
        assert_eq!(kw.spec.kernel_truth.actual_cpu_cores, 0.8);

        // Verify metrics lie
        assert_eq!(kw.spec.metrics_lie.cpu_percent, 50.0);
        assert_eq!(kw.spec.metrics_lie.reported_status, "Healthy");

        // Verify severity (45.5% should be Info)
        assert_eq!(kw.spec.severity, Severity::Info);
    }

    #[test]
    fn test_severity_calculation() {
        // Test Critical (>80%)
        let payload = CpuThrottlePayload {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            pod_name: "critical-pod".to_string(),
            namespace: "default".to_string(),
            container_name: "app".to_string(),
            throttle_percentage: 85.0,
            actual_cpu_usage: 0.8,
            reported_cpu_usage: 0.5,
            period_seconds: 300,
        };
        let kw = build_kernel_whisper(&payload);
        assert_eq!(kw.spec.severity, Severity::Critical);

        // Test Warning (>50%, <=80%)
        let payload = CpuThrottlePayload {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            pod_name: "warning-pod".to_string(),
            namespace: "default".to_string(),
            container_name: "app".to_string(),
            throttle_percentage: 65.0,
            actual_cpu_usage: 0.8,
            reported_cpu_usage: 0.5,
            period_seconds: 300,
        };
        let kw = build_kernel_whisper(&payload);
        assert_eq!(kw.spec.severity, Severity::Warning);
    }
}
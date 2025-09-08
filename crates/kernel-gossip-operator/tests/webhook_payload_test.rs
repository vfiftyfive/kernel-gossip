#[cfg(test)]
mod webhook_payload_tests {
    use kernel_gossip_operator::webhook::EbpfWebhookPayload;
    use serde_json::json;

    #[test]
    fn test_pod_creation_payload_parsing() {
        let payload_json = json!({
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "test-pod",
            "namespace": "default",
            "total_syscalls": 1234,
            "namespace_ops": 56,
            "cgroup_writes": 78,
            "duration_ns": 1000000,
            "timeline": [],
            "ebpf_detection": true
        });

        let payload: EbpfWebhookPayload = serde_json::from_value(payload_json)
            .expect("Failed to parse pod creation payload");

        match payload {
            EbpfWebhookPayload::PodCreation { pod_name, namespace, total_syscalls, namespace_ops, cgroup_writes, duration_ns, ebpf_detection, .. } => {
                assert_eq!(pod_name, "test-pod");
                assert_eq!(namespace, "default");
                assert_eq!(total_syscalls, 1234);
                assert_eq!(namespace_ops, 56);
                assert_eq!(cgroup_writes, 78);
                assert_eq!(duration_ns, 1000000);
                assert!(ebpf_detection);
            }
            _ => panic!("Expected PodCreation payload"),
        }
    }

    #[test]
    fn test_cpu_throttle_payload_parsing() {
        let payload_json = json!({
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "throttled-pod",
            "namespace": "production",
            "container_name": "app",
            "throttle_percentage": 45.5,
            "actual_cpu_usage": 0.8,
            "reported_cpu_usage": 0.5,
            "period_seconds": 300,
            "ebpf_detection": true,
            "throttle_ns": 123456789
        });

        let payload: EbpfWebhookPayload = serde_json::from_value(payload_json)
            .expect("Failed to parse CPU throttle payload");

        match payload {
            EbpfWebhookPayload::CpuThrottle { pod_name, namespace, container_name, throttle_percentage, actual_cpu_usage, reported_cpu_usage, period_seconds, ebpf_detection, throttle_ns, .. } => {
                assert_eq!(pod_name, "throttled-pod");
                assert_eq!(namespace, "production");
                assert_eq!(container_name, "app");
                assert_eq!(throttle_percentage, 45.5);
                assert_eq!(actual_cpu_usage, 0.8);
                assert_eq!(reported_cpu_usage, 0.5);
                assert_eq!(period_seconds, 300);
                assert!(ebpf_detection);
                assert_eq!(throttle_ns, 123456789);
            }
            _ => panic!("Expected CpuThrottle payload"),
        }
    }

    #[test]
    fn test_unknown_payload_type() {
        let payload_json = json!({
            "type": "unknown_type",
            "timestamp": "2024-01-01T00:00:00Z"
        });

        let result: Result<EbpfWebhookPayload, _> = serde_json::from_value(payload_json);
        assert!(result.is_err());
    }
}
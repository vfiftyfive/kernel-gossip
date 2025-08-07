#[cfg(test)]
mod webhook_tests {
    use kernel_gossip_operator::webhook::PixieWebhookPayload;
    use serde_json::json;

    #[test]
    fn test_pod_creation_payload_parsing() {
        let payload_json = json!({
            "type": "pod_creation",
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "test-pod",
            "namespace": "default",
            "total_syscalls": 1234,
            "namespace_ops": 56,
            "cgroup_writes": 78,
            "duration_ns": 1000000
        });

        let payload: PixieWebhookPayload = serde_json::from_value(payload_json)
            .expect("Failed to parse pod creation payload");

        match payload {
            PixieWebhookPayload::PodCreation(data) => {
                assert_eq!(data.pod_name, "test-pod");
                assert_eq!(data.namespace, "default");
                assert_eq!(data.total_syscalls, 1234);
                assert_eq!(data.namespace_ops, 56);
                assert_eq!(data.cgroup_writes, 78);
                assert_eq!(data.duration_ns, 1000000);
            }
            _ => panic!("Expected PodCreation payload"),
        }
    }

    #[test]
    fn test_cpu_throttle_payload_parsing() {
        let payload_json = json!({
            "type": "cpu_throttle",
            "timestamp": "2024-01-01T00:00:00Z",
            "pod_name": "throttled-pod",
            "namespace": "production",
            "container_name": "app",
            "throttle_percentage": 45.5,
            "actual_cpu_usage": 0.8,
            "reported_cpu_usage": 0.5,
            "period_seconds": 300
        });

        let payload: PixieWebhookPayload = serde_json::from_value(payload_json)
            .expect("Failed to parse CPU throttle payload");

        match payload {
            PixieWebhookPayload::CpuThrottle(data) => {
                assert_eq!(data.pod_name, "throttled-pod");
                assert_eq!(data.namespace, "production");
                assert_eq!(data.container_name, "app");
                assert_eq!(data.throttle_percentage, 45.5);
                assert_eq!(data.actual_cpu_usage, 0.8);
                assert_eq!(data.reported_cpu_usage, 0.5);
                assert_eq!(data.period_seconds, 300);
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

        let result: Result<PixieWebhookPayload, _> = serde_json::from_value(payload_json);
        assert!(result.is_err());
    }
}
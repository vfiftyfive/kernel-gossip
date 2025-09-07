pub mod kernel_whisper;
pub mod pod_birth_certificate;

pub use kernel_whisper::*;
pub use pod_birth_certificate::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_birth_certificate_required_fields() {
        // This test MUST FAIL first
        let cert = PodBirthCertificate::create("test-pod", "default");

        assert_eq!(cert.api_version(), "kernel.gossip.io/v1alpha1");
        assert_eq!(cert.kind(), "PodBirthCertificate");
        assert_eq!(cert.pod_name(), "test-pod");
        assert_eq!(cert.namespace(), "default");
    }

    #[test]
    fn test_pod_birth_certificate_serialization() {
        // Test Kubernetes CRD serialization
        let cert = PodBirthCertificate::create("nginx-abc123", "production");

        let json = serde_json::to_value(&cert).expect("serialization failed");

        assert_eq!(json["apiVersion"], "kernel.gossip.io/v1alpha1");
        assert_eq!(json["kind"], "PodBirthCertificate");
        assert_eq!(json["metadata"]["name"], "nginx-abc123-birth");
        assert_eq!(json["spec"]["pod_name"], "nginx-abc123");
    }

    #[test]
    fn test_timeline_entry_creation() {
        let entry = TimelineEntry::new(
            100,
            Actor::Scheduler,
            "Pod assigned to node",
        );

        assert_eq!(entry.timestamp_ms(), 100);
        assert_eq!(entry.actor(), &Actor::Scheduler);
    }

    #[test]
    fn test_kernel_whisper_required_fields() {
        // This test MUST FAIL first
        let whisper = KernelWhisper::create(
            "frontend-xyz789",
            "production",
            85.7, // throttled percent
            45.2, // cpu usage percent
        );

        assert_eq!(whisper.api_version(), "kernel.gossip.io/v1alpha1");
        assert_eq!(whisper.kind(), "KernelWhisper");
        assert_eq!(whisper.pod_name(), "frontend-xyz789");
        assert_eq!(whisper.namespace(), "production");
        assert_eq!(whisper.kernel_truth().throttled_percent, 85.7);
        assert_eq!(whisper.metrics_lie().cpu_percent, 45.2);
    }

    #[test]
    fn test_kernel_whisper_serialization() {
        // Test Kubernetes CRD serialization
        let whisper = KernelWhisper::create("backend-api-123", "staging", 92.3, 38.1);

        let json = serde_json::to_value(&whisper).expect("serialization failed");

        assert_eq!(json["apiVersion"], "kernel.gossip.io/v1alpha1");
        assert_eq!(json["kind"], "KernelWhisper");
        assert_eq!(json["metadata"]["name"], "backend-api-123-cpu-throttle");
        assert_eq!(json["spec"]["pod_name"], "backend-api-123");
        assert_eq!(json["spec"]["kernel_truth"]["throttled_percent"], 92.3);
        assert_eq!(json["spec"]["metrics_lie"]["cpu_percent"], 38.1);
    }

    #[test]
    fn test_severity_enum() {
        let critical = Severity::Critical;
        let warning = Severity::Warning;
        let info = Severity::Info;

        assert_eq!(format!("{critical:?}"), "Critical");
        assert_eq!(format!("{warning:?}"), "Warning");
        assert_eq!(format!("{info:?}"), "Info");
    }
}

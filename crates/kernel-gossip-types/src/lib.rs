pub mod pod_birth_certificate;

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
            Some("node-1 selected based on resources"),
        );

        assert_eq!(entry.timestamp_ms(), 100);
        assert_eq!(entry.actor(), &Actor::Scheduler);
        assert!(entry.details().is_some());
    }
}

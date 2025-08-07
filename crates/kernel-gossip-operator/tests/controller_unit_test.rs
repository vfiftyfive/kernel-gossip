#[cfg(test)]
mod controller_unit_tests {
    use kernel_gossip_operator::crd::{reconcile_logic_pod_birth, reconcile_logic_kernel_whisper};
    use kernel_gossip_types::{PodBirthCertificate, KernelWhisper, Severity};

    #[test]
    fn test_pod_birth_certificate_validation() {
        // Test with valid PodBirthCertificate
        let pbc = PodBirthCertificate::create("test-pod", "default");
        let result = reconcile_logic_pod_birth(&pbc);
        assert!(result.is_ok());
        
        // Test with empty name
        let mut invalid_pbc = PodBirthCertificate::create("", "default");
        invalid_pbc.metadata.name = Some("".to_string());
        let result = reconcile_logic_pod_birth(&invalid_pbc);
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_whisper_severity_logic() {
        // Test Critical severity
        let mut kw = KernelWhisper::create("critical-pod", "default", 85.0, 80.0);
        kw.spec.severity = Severity::Critical;
        let action = reconcile_logic_kernel_whisper(&kw);
        assert_eq!(action.severity_level(), "critical");
        assert!(action.requires_remediation());
        
        // Test Warning severity
        let mut kw = KernelWhisper::create("warning-pod", "default", 65.0, 70.0);
        kw.spec.severity = Severity::Warning;
        let action = reconcile_logic_kernel_whisper(&kw);
        assert_eq!(action.severity_level(), "warning");
        assert!(!action.requires_remediation());
        
        // Test Info severity
        let mut kw = KernelWhisper::create("info-pod", "default", 15.0, 50.0);
        kw.spec.severity = Severity::Info;
        let action = reconcile_logic_kernel_whisper(&kw);
        assert_eq!(action.severity_level(), "info");
        assert!(!action.requires_remediation());
    }

    #[test]
    fn test_requeue_duration_calculation() {
        use kernel_gossip_operator::crd::calculate_requeue_duration;
        
        // Critical should requeue quickly
        assert_eq!(calculate_requeue_duration(&Severity::Critical), 60);
        
        // Warning should requeue moderately
        assert_eq!(calculate_requeue_duration(&Severity::Warning), 180);
        
        // Info should requeue slowly
        assert_eq!(calculate_requeue_duration(&Severity::Info), 600);
    }
}
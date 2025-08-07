#[cfg(test)]
mod config_tests {
    use kernel_gossip_operator::config::Config;

    #[test]
    fn test_config_from_env() {
        // Test that we can load config from environment variables
        std::env::set_var("WEBHOOK_PORT", "8080");
        std::env::set_var("METRICS_PORT", "9090");
        std::env::set_var("PIXIE_API_KEY", "test-key");
        std::env::set_var("PIXIE_CLUSTER_ID", "test-cluster");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.webhook_port, 8080);
        assert_eq!(config.metrics_port, 9090);
        assert_eq!(config.pixie_api_key, "test-key");
        assert_eq!(config.pixie_cluster_id, "test-cluster");
    }

    #[test]
    fn test_config_default_values() {
        // Set required vars but leave optional ones unset
        std::env::set_var("PIXIE_API_KEY", "test-key-default");
        std::env::set_var("PIXIE_CLUSTER_ID", "test-cluster-default");
        // Don't set WEBHOOK_PORT or METRICS_PORT to test defaults

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.webhook_port, 8080); // default
        assert_eq!(config.metrics_port, 9090); // default
        assert_eq!(config.pixie_api_key, "test-key-default");
        assert_eq!(config.pixie_cluster_id, "test-cluster-default");
    }

    #[test]
    fn test_config_missing_required() {
        // Test that missing required fields cause error
        std::env::remove_var("PIXIE_API_KEY");
        std::env::remove_var("PIXIE_CLUSTER_ID");

        let result = Config::from_env();
        assert!(result.is_err());
    }
}
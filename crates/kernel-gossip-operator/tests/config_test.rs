#[cfg(test)]
mod config_tests {
    use kernel_gossip_operator::config::Config;

    #[test]
    fn test_config_from_env() {
        // Test that we can load config from environment variables
        std::env::set_var("WEBHOOK_PORT", "8080");
        std::env::set_var("METRICS_PORT", "9090");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.webhook_port, 8080);
        assert_eq!(config.metrics_port, 9090);
    }

    #[test]
    fn test_config_default_values() {
        // Don't set WEBHOOK_PORT or METRICS_PORT to test defaults
        std::env::remove_var("WEBHOOK_PORT");
        std::env::remove_var("METRICS_PORT");

        let config = Config::from_env().expect("Failed to load config");

        assert_eq!(config.webhook_port, 8080); // default
        assert_eq!(config.metrics_port, 9090); // default
    }

    #[test]
    fn test_config_always_succeeds() {
        // Config should always succeed with defaults
        std::env::remove_var("WEBHOOK_PORT");
        std::env::remove_var("METRICS_PORT");

        let result = Config::from_env();
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.webhook_port, 8080);
        assert_eq!(config.metrics_port, 9090);
    }
}
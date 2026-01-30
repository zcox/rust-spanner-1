use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub spanner_emulator_host: Option<String>,
    pub spanner_project: String,
    pub spanner_instance: String,
    pub spanner_database: String,
    pub service_port: u16,
    pub service_host: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let spanner_emulator_host = env::var("SPANNER_EMULATOR_HOST").ok();

        let spanner_project = env::var("SPANNER_PROJECT")
            .context("SPANNER_PROJECT environment variable is required")?;

        let spanner_instance = env::var("SPANNER_INSTANCE")
            .context("SPANNER_INSTANCE environment variable is required")?;

        let spanner_database = env::var("SPANNER_DATABASE")
            .context("SPANNER_DATABASE environment variable is required")?;

        let service_port = env::var("SERVICE_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("SERVICE_PORT must be a valid port number (0-65535)")?;

        let service_host = env::var("SERVICE_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        Ok(Config {
            spanner_emulator_host,
            spanner_project,
            spanner_instance,
            spanner_database,
            service_port,
            service_host,
        })
    }

    pub fn log_startup(&self) {
        tracing::info!("Configuration loaded:");
        tracing::info!("  Spanner emulator: {}",
            self.spanner_emulator_host.as_deref().unwrap_or("disabled (using production)"));
        tracing::info!("  Spanner project: {}", self.spanner_project);
        tracing::info!("  Spanner instance: {}", self.spanner_instance);
        tracing::info!("  Spanner database: {}", self.spanner_database);
        tracing::info!("  Service listening on: {}:{}", self.service_host, self.service_port);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn clear_env_vars() {
        unsafe {
            env::remove_var("SPANNER_EMULATOR_HOST");
            env::remove_var("SPANNER_PROJECT");
            env::remove_var("SPANNER_INSTANCE");
            env::remove_var("SPANNER_DATABASE");
            env::remove_var("SERVICE_PORT");
            env::remove_var("SERVICE_HOST");
        }
    }

    fn set_required_vars() {
        unsafe {
            env::set_var("SPANNER_PROJECT", "test-project");
            env::set_var("SPANNER_INSTANCE", "test-instance");
            env::set_var("SPANNER_DATABASE", "test-database");
        }
    }

    #[test]
    fn test_config_with_all_vars() {
        clear_env_vars();
        set_required_vars();
        unsafe {
            env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
            env::set_var("SERVICE_PORT", "8080");
            env::set_var("SERVICE_HOST", "127.0.0.1");
        }

        let config = Config::from_env().unwrap();

        assert_eq!(config.spanner_emulator_host, Some("localhost:9010".to_string()));
        assert_eq!(config.spanner_project, "test-project");
        assert_eq!(config.spanner_instance, "test-instance");
        assert_eq!(config.spanner_database, "test-database");
        assert_eq!(config.service_port, 8080);
        assert_eq!(config.service_host, "127.0.0.1");
    }

    #[test]
    fn test_config_with_defaults() {
        clear_env_vars();
        set_required_vars();

        let config = Config::from_env().unwrap();

        assert_eq!(config.spanner_emulator_host, None);
        assert_eq!(config.service_port, 3000);
        assert_eq!(config.service_host, "0.0.0.0");
    }

    #[test]
    fn test_missing_required_var() {
        clear_env_vars();
        unsafe {
            env::set_var("SPANNER_PROJECT", "test-project");
            env::set_var("SPANNER_INSTANCE", "test-instance");
        }
        // Missing SPANNER_DATABASE

        let result = Config::from_env();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("SPANNER_DATABASE"));
    }

    #[test]
    fn test_invalid_port() {
        clear_env_vars();
        set_required_vars();
        unsafe {
            env::set_var("SERVICE_PORT", "not-a-number");
        }

        let result = Config::from_env();
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("SERVICE_PORT"));
    }

    #[test]
    fn test_port_out_of_range() {
        clear_env_vars();
        set_required_vars();
        unsafe {
            env::set_var("SERVICE_PORT", "99999");
        }

        let result = Config::from_env();
        assert!(result.is_err());
    }
}

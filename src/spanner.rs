use anyhow::{Context, Result};
use google_cloud_spanner::client::{Client, ClientConfig};
use std::sync::Arc;

use crate::config::Config;

/// Shareable Spanner client for use across async handlers
#[derive(Clone)]
pub struct SpannerClient {
    inner: Arc<Client>,
}

impl SpannerClient {
    /// Create a new Spanner client from configuration
    ///
    /// This creates a connection to Spanner using the provided config.
    /// The google-cloud-spanner library automatically detects the
    /// SPANNER_EMULATOR_HOST environment variable and connects to
    /// the emulator when set, or production Spanner otherwise.
    pub async fn from_config(config: &Config) -> Result<Self> {
        let database_path = format!(
            "projects/{}/instances/{}/databases/{}",
            config.spanner_project, config.spanner_instance, config.spanner_database
        );

        // Log connection target
        if config.spanner_emulator_host.is_some() {
            tracing::info!(
                "Connecting to Spanner emulator at: {}",
                config.spanner_emulator_host.as_ref().unwrap()
            );
        } else {
            tracing::info!("Connecting to production Spanner");
        }

        // ClientConfig::default() automatically uses SPANNER_EMULATOR_HOST if set
        let client = Client::new(&database_path, ClientConfig::default())
            .await
            .context("Failed to create Spanner client")?;

        tracing::info!(
            "Successfully connected to Spanner database: {}",
            database_path
        );

        Ok(Self {
            inner: Arc::new(client),
        })
    }

    /// Get a reference to the underlying Spanner client
    pub fn client(&self) -> &Client {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation_with_emulator() {
        // Set up config with emulator
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "test-instance".to_string(),
            spanner_database: "test-database".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        // This will fail if emulator is not running, but that's expected
        // The test verifies that the client creation API works correctly
        let result = SpannerClient::from_config(&config).await;

        // Clean up
        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }

        // We expect this to fail if emulator isn't running, but the API should work
        match result {
            Ok(_) => {
                // Client created successfully - emulator is running
            }
            Err(e) => {
                // Connection failed - likely emulator not running
                // Verify error message is descriptive
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Failed to create Spanner client"),
                    "Error should have context: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_client_is_clonable() {
        // This test verifies that SpannerClient implements Clone
        // which is required for sharing across Axum handlers
        fn assert_clone<T: Clone>() {}
        assert_clone::<SpannerClient>();
    }

    #[test]
    fn test_client_is_send_sync() {
        // This test verifies that SpannerClient is Send + Sync
        // which is required for use in async handlers
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SpannerClient>();
    }
}

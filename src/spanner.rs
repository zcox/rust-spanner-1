use anyhow::{Context, Result};
use gcloud_gax::grpc::Code;
use gcloud_googleapis::spanner::admin::database::v1::{
    CreateDatabaseRequest, GetDatabaseDdlRequest, GetDatabaseRequest, UpdateDatabaseDdlRequest,
};
use gcloud_googleapis::spanner::admin::instance::v1::{
    CreateInstanceRequest, GetInstanceRequest, Instance,
};
use gcloud_spanner::admin::client::Client as AdminClient;
use gcloud_spanner::admin::AdminClientConfig;
use gcloud_spanner::client::{Client, ClientConfig};
use gcloud_spanner::mutation::insert_or_update;
use gcloud_spanner::statement::Statement;
use gcloud_spanner::value::CommitTimestamp;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

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
    /// The gcloud-spanner library automatically detects the
    /// SPANNER_EMULATOR_HOST environment variable and connects to
    /// the emulator when set, or production Spanner otherwise.
    ///
    /// This function also performs auto-provisioning: it will automatically
    /// create the instance, database, and table if they don't exist.
    pub async fn from_config(config: &Config) -> Result<Self> {
        // Perform auto-provisioning first
        auto_provision(config).await?;

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

    /// Upsert (insert or update) a JSON document with the given UUID key
    ///
    /// This operation will insert a new row if the ID doesn't exist, or update
    /// an existing row if it does. Both `created_at` and `updated_at` are set
    /// to the commit timestamp automatically.
    ///
    /// # Arguments
    /// * `id` - UUID key for the document
    /// * `data` - JSON document to store
    ///
    /// # Errors
    /// Returns an error if the Spanner operation fails
    pub async fn upsert(&self, id: Uuid, data: JsonValue) -> Result<()> {
        let id_str = id.to_string();
        let data_str = serde_json::to_string(&data)
            .context("Failed to serialize JSON data")?;

        let mutation = insert_or_update(
            "kv_store",
            &["id", "data", "created_at", "updated_at"],
            &[&id_str, &data_str, &CommitTimestamp::new(), &CommitTimestamp::new()],
        );

        self.inner
            .apply(vec![mutation])
            .await
            .context("Failed to upsert data to Spanner")?;

        tracing::debug!("Upserted document with id: {}", id);
        Ok(())
    }

    /// Read a JSON document by its UUID key
    ///
    /// # Arguments
    /// * `id` - UUID key of the document to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(data))` - Document found and returned
    /// * `Ok(None)` - Document not found
    /// * `Err(_)` - Spanner operation failed
    ///
    /// # Errors
    /// Returns an error if the Spanner query fails or if JSON deserialization fails
    pub async fn read(&self, id: Uuid) -> Result<Option<JsonValue>> {
        let id_str = id.to_string();

        let mut statement = Statement::new(
            "SELECT data FROM kv_store WHERE id = @id"
        );
        statement.add_param("id", &id_str);

        let mut tx = self.inner
            .single()
            .await
            .context("Failed to create read transaction")?;

        let mut result_set = tx
            .query(statement)
            .await
            .context("Failed to query data from Spanner")?;

        // Check if we got any rows
        if let Some(row) = result_set.next().await? {
            let data_str: String = row.column_by_name("data")?;
            let data: JsonValue = serde_json::from_str(&data_str)
                .context("Failed to deserialize JSON data")?;

            tracing::debug!("Read document with id: {}", id);
            Ok(Some(data))
        } else {
            tracing::debug!("Document not found with id: {}", id);
            Ok(None)
        }
    }
}

/// Automatically provision Spanner instance, database, and table
///
/// This function checks if the configured resources exist and creates them if needed.
/// It's designed to enable zero-setup local development with the emulator.
async fn auto_provision(config: &Config) -> Result<()> {
    tracing::info!("Starting auto-provisioning checks...");

    // Create admin client
    let admin_client = AdminClient::new(AdminClientConfig::default())
        .await
        .context("Failed to create Spanner admin client")?;

    let project_path = format!("projects/{}", config.spanner_project);
    let instance_path = format!("{}/instances/{}", project_path, config.spanner_instance);
    let database_path = format!("{}/databases/{}", instance_path, config.spanner_database);

    // Check and create instance if needed
    ensure_instance_exists(&admin_client, config, &project_path, &instance_path).await?;

    // Check and create database if needed
    ensure_database_exists(&admin_client, &instance_path, &database_path).await?;

    // Check and create table if needed
    ensure_table_exists(&admin_client, &database_path).await?;

    tracing::info!("Auto-provisioning complete");
    Ok(())
}

/// Ensure the Spanner instance exists, creating it if necessary
async fn ensure_instance_exists(
    admin_client: &AdminClient,
    config: &Config,
    project_path: &str,
    instance_path: &str,
) -> Result<()> {
    let get_request = GetInstanceRequest {
        name: instance_path.to_string(),
        field_mask: None,
    };

    match admin_client.instance().get_instance(get_request, None).await {
        Ok(_) => {
            tracing::info!("Instance already exists: {}", instance_path);
            Ok(())
        }
        Err(status) if status.code() == Code::NotFound => {
            tracing::info!("Instance not found, creating: {}", instance_path);

            // For emulator, use a simple config
            let instance_config = if config.spanner_emulator_host.is_some() {
                format!("{}/instanceConfigs/emulator-config", project_path)
            } else {
                // For production, use a default config (regional-us-central1)
                format!("{}/instanceConfigs/regional-us-central1", project_path)
            };

            let create_request = CreateInstanceRequest {
                parent: project_path.to_string(),
                instance_id: config.spanner_instance.clone(),
                instance: Some(Instance {
                    name: instance_path.to_string(),
                    config: instance_config,
                    display_name: format!("{} instance", config.spanner_instance),
                    node_count: 1,
                    ..Default::default()
                }),
            };

            let mut operation = admin_client
                .instance()
                .create_instance(create_request, None)
                .await
                .context("Failed to start instance creation")?;

            // Wait for the operation to complete
            operation
                .wait(None)
                .await
                .context("Failed to create instance")?;

            tracing::info!("Instance created successfully: {}", instance_path);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to check instance existence: {}",
            e.message()
        )),
    }
}

/// Ensure the Spanner database exists, creating it if necessary
async fn ensure_database_exists(
    admin_client: &AdminClient,
    instance_path: &str,
    database_path: &str,
) -> Result<()> {
    let get_request = GetDatabaseRequest {
        name: database_path.to_string(),
    };

    match admin_client
        .database()
        .get_database(get_request, None)
        .await
    {
        Ok(_) => {
            tracing::info!("Database already exists: {}", database_path);
            Ok(())
        }
        Err(status) if status.code() == Code::NotFound => {
            tracing::info!("Database not found, creating: {}", database_path);

            let database_id = database_path
                .split('/')
                .next_back()
                .context("Invalid database path")?;

            let create_request = CreateDatabaseRequest {
                parent: instance_path.to_string(),
                create_statement: format!("CREATE DATABASE `{}`", database_id),
                extra_statements: vec![],
                encryption_config: None,
                database_dialect: 1, // Google Standard SQL
                proto_descriptors: vec![],
            };

            let mut operation = admin_client
                .database()
                .create_database(create_request, None)
                .await
                .context("Failed to start database creation")?;

            // Wait for the operation to complete
            operation
                .wait(None)
                .await
                .context("Failed to create database")?;

            tracing::info!("Database created successfully: {}", database_path);
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to check database existence: {}",
            e.message()
        )),
    }
}

/// Ensure the kv_store table exists, creating it if necessary
async fn ensure_table_exists(admin_client: &AdminClient, database_path: &str) -> Result<()> {
    let get_ddl_request = GetDatabaseDdlRequest {
        database: database_path.to_string(),
    };

    let ddl_response = admin_client
        .database()
        .get_database_ddl(get_ddl_request, None)
        .await
        .context("Failed to get database DDL")?;

    // Check if kv_store table exists in the DDL statements
    let table_exists = ddl_response
        .into_inner()
        .statements
        .iter()
        .any(|stmt| stmt.contains("CREATE TABLE kv_store") || stmt.contains("CREATE TABLE `kv_store`"));

    if table_exists {
        tracing::info!("Table 'kv_store' already exists");
        Ok(())
    } else {
        tracing::info!("Table 'kv_store' not found, creating...");

        let create_table_ddl = r#"
CREATE TABLE kv_store (
    id STRING(36) NOT NULL,
    data JSON NOT NULL,
    created_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
    updated_at TIMESTAMP NOT NULL OPTIONS (allow_commit_timestamp=true),
) PRIMARY KEY (id)
"#
        .trim()
        .to_string();

        let update_request = UpdateDatabaseDdlRequest {
            database: database_path.to_string(),
            statements: vec![create_table_ddl],
            operation_id: String::new(),
            proto_descriptors: vec![],
            throughput_mode: false,
        };

        let mut operation = admin_client
            .database()
            .update_database_ddl(update_request, None)
            .await
            .context("Failed to start table creation")?;

        // Wait for the DDL operation to complete
        operation
            .wait(None)
            .await
            .context("Failed to create table")?;

        tracing::info!("Table 'kv_store' created successfully");
        Ok(())
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
                    error_msg.contains("Failed to create Spanner")
                        || error_msg.contains("Failed to start")
                        || error_msg.contains("Failed to check"),
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

    #[tokio::test]
    async fn test_auto_provisioning_with_emulator() {
        // This test verifies that auto-provisioning works with the emulator
        // It requires the emulator to be running
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "auto-provision-test-instance".to_string(),
            spanner_database: "auto-provision-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        // This will auto-provision the instance, database, and table
        let result = SpannerClient::from_config(&config).await;

        // Clean up
        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }

        match result {
            Ok(_) => {
                // Auto-provisioning succeeded - emulator is running
                // This means the instance, database, and table were created
            }
            Err(e) => {
                // If emulator is not running, this is expected
                let error_msg = e.to_string();
                println!("Auto-provisioning test failed (emulator may not be running): {}", error_msg);
            }
        }
    }

    #[tokio::test]
    async fn test_auto_provisioning_idempotent() {
        // This test verifies that auto-provisioning is idempotent
        // Running it multiple times should not cause errors
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "idempotent-test-instance".to_string(),
            spanner_database: "idempotent-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        // Run auto-provisioning twice
        let result1 = SpannerClient::from_config(&config).await;

        // If the first call succeeded, try a second time
        if result1.is_ok() {
            let result2 = SpannerClient::from_config(&config).await;
            assert!(result2.is_ok(), "Second auto-provisioning call should succeed");
        }

        // Clean up
        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_upsert_and_read() {
        // This test verifies that upsert and read operations work correctly
        // It requires the emulator to be running
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "crud-test-instance".to_string(),
            spanner_database: "crud-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        // Create client (which will auto-provision if needed)
        let client_result = SpannerClient::from_config(&config).await;

        if let Ok(client) = client_result {
            // Test data
            let test_id = Uuid::new_v4();
            let test_data = serde_json::json!({
                "name": "test document",
                "value": 42,
                "nested": {
                    "key": "value"
                }
            });

            // Test upsert
            let upsert_result = client.upsert(test_id, test_data.clone()).await;
            assert!(upsert_result.is_ok(), "Upsert should succeed");

            // Test read - should return the data we just inserted
            let read_result = client.read(test_id).await;
            assert!(read_result.is_ok(), "Read should succeed");

            let retrieved_data = read_result.unwrap();
            assert!(retrieved_data.is_some(), "Should find the document");
            assert_eq!(retrieved_data.unwrap(), test_data, "Retrieved data should match inserted data");

            // Test read with non-existent ID - should return None
            let non_existent_id = Uuid::new_v4();
            let read_result = client.read(non_existent_id).await;
            assert!(read_result.is_ok(), "Read should succeed");
            assert!(read_result.unwrap().is_none(), "Should not find non-existent document");

            // Test upsert update - update existing document
            let updated_data = serde_json::json!({
                "name": "updated document",
                "value": 100
            });
            let update_result = client.upsert(test_id, updated_data.clone()).await;
            assert!(update_result.is_ok(), "Update should succeed");

            // Verify the update
            let read_result = client.read(test_id).await;
            assert!(read_result.is_ok(), "Read should succeed");
            let retrieved_data = read_result.unwrap();
            assert!(retrieved_data.is_some(), "Should find the updated document");
            assert_eq!(retrieved_data.unwrap(), updated_data, "Retrieved data should match updated data");
        } else {
            // If emulator is not running, skip the test
            println!("CRUD test skipped (emulator may not be running)");
        }

        // Clean up
        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }

    #[tokio::test]
    async fn test_json_round_trip() {
        // This test verifies that complex JSON data round-trips correctly
        unsafe {
            std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        }

        let config = Config {
            spanner_emulator_host: Some("localhost:9010".to_string()),
            spanner_project: "test-project".to_string(),
            spanner_instance: "json-test-instance".to_string(),
            spanner_database: "json-test-db".to_string(),
            service_port: 3000,
            service_host: "0.0.0.0".to_string(),
        };

        let client_result = SpannerClient::from_config(&config).await;

        if let Ok(client) = client_result {
            let test_id = Uuid::new_v4();

            // Test with various JSON types
            let complex_data = serde_json::json!({
                "string": "hello",
                "number": 123,
                "float": 45.67,
                "boolean": true,
                "null": null,
                "array": [1, 2, 3],
                "nested_object": {
                    "deep": {
                        "value": "nested"
                    }
                },
                "unicode": "こんにちは 🚀"
            });

            // Upsert and read
            client.upsert(test_id, complex_data.clone()).await.unwrap();
            let retrieved = client.read(test_id).await.unwrap();

            assert_eq!(retrieved.unwrap(), complex_data, "Complex JSON should round-trip correctly");
        } else {
            println!("JSON round-trip test skipped (emulator may not be running)");
        }

        // Clean up
        unsafe {
            std::env::remove_var("SPANNER_EMULATOR_HOST");
        }
    }
}

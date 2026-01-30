mod config;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("rust-spanner-kv starting");

    let config = Config::from_env()?;
    config.log_startup();

    Ok(())
}

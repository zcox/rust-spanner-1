mod config;
mod spanner;

use config::Config;
use spanner::SpannerClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("rust-spanner-kv starting");

    let config = Config::from_env()?;
    config.log_startup();

    let _spanner_client = SpannerClient::from_config(&config).await?;

    Ok(())
}

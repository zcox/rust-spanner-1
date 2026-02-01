use crate::config::Config;
use crate::spanner::SpannerClient;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub spanner_client: SpannerClient,
    pub config: Arc<Config>,
}

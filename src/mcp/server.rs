//! axum listener: `/mcp` (Streamable HTTP) and `/healthz`.

use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
};

use crate::index::IndexStore;
use crate::retrieval::SearchService;

use super::dto::{FailedEditionHealth, Health};
use super::tools::LesefruchtServer;

#[derive(Clone)]
struct HealthState {
    store: Arc<IndexStore>,
}

pub fn router(service: Arc<SearchService>, store: Arc<IndexStore>, mcp_path: &str) -> Router {
    let mcp_service = StreamableHttpService::new(
        {
            let service = Arc::clone(&service);
            move || Ok(LesefruchtServer::new(Arc::clone(&service)))
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    Router::new()
        .route("/healthz", get(health))
        .nest_service(mcp_path, mcp_service)
        .with_state(HealthState { store })
}

async fn health(State(state): State<HealthState>) -> Json<Health> {
    let status = state.store.status().ok();
    let (indexed_editions, failed_editions, last_success_at, library_generation, index_state) =
        match status {
            Some(status) => (
                status.indexed_editions,
                status
                    .failed_editions
                    .into_iter()
                    .map(|row| FailedEditionHealth {
                        work_id: row.work_id.0,
                        edition_id: row.edition_id.0,
                        format: row.format,
                        reason: row.reason,
                    })
                    .collect(),
                status.last_success_at,
                status.library_generation,
                status.state,
            ),
            None => (0, Vec::new(), None, None, "unavailable".into()),
        };
    Json(Health {
        indexed_editions,
        failed_editions,
        last_success_at,
        library_generation,
        index_state,
    })
}

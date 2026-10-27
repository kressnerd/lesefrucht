//! One consumer owns the indexing pass; the store serialises writes.

use std::sync::Arc;

use crate::config::Config;
use crate::index::IndexStore;

use super::pipeline::Indexer;

pub fn spawn_initial_indexing(
    store: Arc<IndexStore>,
    config: Config,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let indexer = Indexer::new(store, config);
        if let Err(error) = indexer.run().await {
            tracing::warn!(%error, "initial indexing pass failed");
        }
    })
}

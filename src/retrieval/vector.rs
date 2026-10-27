//! Vector retrieval path.

use std::collections::HashSet;

use crate::index::vector_index::VectorHit;
use crate::index::{IndexStore, StoreError};
use crate::indexing::embed::{EmbedError, EmbeddingClient};

#[derive(Debug, thiserror::Error)]
pub enum VectorPathError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Embed(#[from] EmbedError),
}

pub async fn search(
    store: &IndexStore,
    embedder: &EmbeddingClient,
    query: &str,
    allowed: Option<&HashSet<String>>,
    limit: usize,
) -> Result<Vec<VectorHit>, VectorPathError> {
    let vector = embedder
        .embed(&[query.to_string()])
        .await?
        .into_iter()
        .next()
        .unwrap_or_default();
    Ok(store.vector_search(&vector, allowed, limit)?)
}

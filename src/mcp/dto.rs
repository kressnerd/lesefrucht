//! Request and response types exposed by the two tools.

pub use crate::retrieval::scope::Scope;
pub use crate::retrieval::{ReadRequest, SearchRequest};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FailedEditionHealth {
    pub work_id: String,
    pub edition_id: String,
    pub format: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct Health {
    pub indexed_editions: usize,
    pub failed_editions: Vec<FailedEditionHealth>,
    pub last_success_at: Option<u64>,
    pub library_generation: Option<String>,
    pub index_state: String,
}

//! The derived index: SQLite metadata, tantivy lexical, usearch vectors, text.

pub mod lexical_index;
pub mod sqlite;
pub mod store;
pub mod text_store;
pub mod vector_index;

pub use store::{now_ms, IndexStore, PassageRecord, StoreError, StoreStatus};

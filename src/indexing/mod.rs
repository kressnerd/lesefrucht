//! The enabling subfunction: turn editions into passages in the index.

pub mod chunk;
pub mod embed;
pub mod extract;
pub mod pipeline;
pub mod queue;
pub mod state;

pub use pipeline::{Indexer, PipelineError};
pub use queue::spawn_initial_indexing;
pub use state::{freshness_ok, StoreState};

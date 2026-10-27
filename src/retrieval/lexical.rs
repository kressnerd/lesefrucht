//! Lexical retrieval path.

use std::collections::HashSet;

use crate::index::lexical_index::LexicalHit;
use crate::index::{IndexStore, StoreError};

pub fn search(
    store: &IndexStore,
    tokens: &[String],
    allowed: Option<&HashSet<String>>,
    limit: usize,
) -> Result<Vec<LexicalHit>, StoreError> {
    store.lexical_search(tokens, allowed, limit)
}

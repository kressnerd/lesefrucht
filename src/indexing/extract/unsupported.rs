//! AZW3, MOBI and anything else: explicit failure, never a silent skip.

use crate::domain::Chapter;

use super::{ExtractError, Extractor};

pub struct UnsupportedExtractor;

impl Extractor for UnsupportedExtractor {
    fn extract(&self, _bytes: &[u8]) -> Result<Vec<Chapter>, ExtractError> {
        Err(ExtractError::Unsupported)
    }
}

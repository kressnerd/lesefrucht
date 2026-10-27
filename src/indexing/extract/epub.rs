//! EPUB 2/3 via rbook, in spine order.

use std::io::Cursor;

use crate::domain::Chapter;

use super::{ExtractError, Extractor, html_to_text};

pub struct EpubExtractor;

impl Extractor for EpubExtractor {
    fn extract(&self, bytes: &[u8]) -> Result<Vec<Chapter>, ExtractError> {
        let epub = rbook::Epub::read(Cursor::new(bytes.to_vec()))
            .map_err(|error| ExtractError::Failed(format!("epub: {error}")))?;
        let mut chapters = Vec::new();
        for (index, entry) in epub.spine().iter().enumerate() {
            let Some(manifest) = entry.manifest_entry() else {
                continue;
            };
            let html = manifest
                .read_str()
                .map_err(|error| ExtractError::Failed(format!("epub resource: {error}")))?;
            let text = html_to_text(&html);
            if text.trim().is_empty() {
                continue;
            }
            let title = manifest
                .href()
                .to_string()
                .rsplit('/')
                .next()
                .map(ToOwned::to_owned);
            chapters.push(Chapter {
                index: index as u16,
                title,
                text,
            });
        }
        if chapters.is_empty() {
            return Err(ExtractError::EmptyText);
        }
        for (position, chapter) in chapters.iter_mut().enumerate() {
            chapter.index = position as u16;
        }
        Ok(chapters)
    }
}

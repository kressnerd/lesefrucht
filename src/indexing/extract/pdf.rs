//! PDF via pdf-extract, one chapter per page group.

use crate::domain::Chapter;

use super::{ExtractError, Extractor};

pub struct PdfExtractor;

impl Extractor for PdfExtractor {
    fn extract(&self, bytes: &[u8]) -> Result<Vec<Chapter>, ExtractError> {
        let pages = pdf_extract::extract_text_from_mem_by_pages(bytes)
            .map_err(|error| ExtractError::Failed(format!("pdf: {error}")))?;
        let mut chapters = Vec::new();
        for (index, page) in pages.into_iter().enumerate() {
            let text = page.trim().to_string();
            if text.is_empty() {
                continue;
            }
            chapters.push(Chapter {
                index: index as u16,
                title: Some(format!("Page {}", index + 1)),
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

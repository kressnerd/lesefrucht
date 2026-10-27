//! Turn a calibre listing into works and edition candidates.

use sha2::{Digest, Sha256};

use crate::domain::{
    Edition, EditionId, Genre, IndexState, Work, WorkId,
};

use super::calibredb::ListingBook;
use super::selection::select_format;

#[derive(Debug, Clone)]
pub struct LibrarySnapshot {
    pub works: Vec<Work>,
    pub editions: Vec<Edition>,
    pub generation: String,
}

pub fn discovery(
    books: &[ListingBook],
    format_preference: &[String],
) -> LibrarySnapshot {
    let mut works = Vec::with_capacity(books.len());
    let mut editions = Vec::new();
    let mut generation_parts = Vec::new();

    for book in books {
        let work_id = WorkId(book.id.to_string());
        let formats = normalized_formats(book);
        let selected = select_format(&formats, format_preference);
        let authors = split_authors(book.authors.as_deref());
        let language = map_language(book.languages.as_deref());
        let genre = book
            .tags
            .as_deref()
            .unwrap_or_default()
            .iter()
            .find_map(|tag| Genre::from_tag(tag));

        works.push(Work {
            id: work_id.clone(),
            title: if book.title.is_empty() {
                format!("Untitled {}", work_id.0)
            } else {
                book.title.clone()
            },
            authors: authors.clone(),
            language: language.clone(),
            genre,
            pubdate: book.pubdate.clone(),
        });

        for format in &formats {
            editions.push(Edition {
                id: EditionId::new(&work_id, format),
                work_id: work_id.clone(),
                format: format.clone(),
                content_hash: None,
                selected: selected.as_deref() == Some(format.as_str()),
                index_state: IndexState::NotIndexed,
                last_checked_at: None,
            });
            generation_parts.push(format!(
                "{}|{}|{}",
                work_id.0,
                format,
                book.last_modified.as_deref().unwrap_or("")
            ));
        }
    }

    generation_parts.sort();
    let mut hasher = Sha256::new();
    for part in &generation_parts {
        hasher.update(part.as_bytes());
        hasher.update(b"\n");
    }
    LibrarySnapshot {
        works,
        editions,
        generation: hex::encode(hasher.finalize()),
    }
}

fn normalized_formats(book: &ListingBook) -> Vec<String> {
    let mut formats: Vec<String> = book
        .formats
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|format| {
            std::path::Path::new(format)
                .extension()
                .and_then(|value| value.to_str())
                .map(|value| value.to_ascii_lowercase())
                .unwrap_or_else(|| format.to_ascii_lowercase())
        })
        .collect();
    formats.sort();
    formats.dedup();
    formats
}

pub fn split_authors(authors: Option<&str>) -> Vec<String> {
    authors
        .unwrap_or_default()
        .split(" & ")
        .map(str::trim)
        .filter(|author| !author.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn map_language(languages: Option<&[String]>) -> String {
    let Some(code) = languages.and_then(|values| values.first()) else {
        return "und".to_string();
    };
    let lower = code.to_ascii_lowercase();
    match lower.as_str() {
        "deu" | "ger" | "de" => "de".into(),
        "eng" | "en" => "en".into(),
        "fra" | "fre" | "fr" => "fr".into(),
        "spa" | "es" => "es".into(),
        "ita" | "it" => "it".into(),
        other if other.len() >= 2 => other[..2].to_string(),
        other => other.to_string(),
    }
}

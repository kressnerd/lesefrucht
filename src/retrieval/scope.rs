//! Scope -> allowed work ids, applied before fusion.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::{Genre, Work, WorkId};

#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct Scope {
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub year_from: Option<i32>,
    #[serde(default)]
    pub year_to: Option<i32>,
    #[serde(default)]
    pub work_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScopeError {
    #[error("unknown work")]
    UnknownWork,
    #[error("invalid work id")]
    InvalidWorkId,
    #[error("invalid genre")]
    InvalidGenre,
}

/// Returns the allowed work ids. An empty set means the scope admits nothing.
pub fn resolve(scope: &Scope, works: &[Work]) -> Result<HashSet<WorkId>, ScopeError> {
    let named: Option<HashSet<WorkId>> = if scope.work_ids.is_empty() {
        None
    } else {
        let mut ids = HashSet::new();
        for raw in &scope.work_ids {
            if raw.parse::<u64>().is_err() {
                return Err(ScopeError::InvalidWorkId);
            }
            let id = WorkId(raw.clone());
            if !works.iter().any(|work| work.id == id) {
                return Err(ScopeError::UnknownWork);
            }
            ids.insert(id);
        }
        Some(ids)
    };
    let genre = match scope.genre.as_deref() {
        Some(value) => Some(parse_genre(value)?),
        None => None,
    };
    let allowed: HashSet<WorkId> = works
        .iter()
        .filter(|work| named.as_ref().is_none_or(|ids| ids.contains(&work.id)))
        .filter(|work| genre.is_none_or(|genre| work.genre == Some(genre)))
        .filter(|work| {
            scope
                .language
                .as_deref()
                .is_none_or(|language| work.language.eq_ignore_ascii_case(language))
        })
        .filter(|work| {
            scope.authors.is_empty()
                || scope.authors.iter().any(|author| {
                    work.authors
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(author))
                })
        })
        .filter(|work| {
            let year = work
                .pubdate
                .as_deref()
                .and_then(|value| value.get(..4))
                .and_then(|value| value.parse::<i32>().ok());
            match year {
                Some(year) => {
                    scope.year_from.is_none_or(|from| year >= from)
                        && scope.year_to.is_none_or(|to| year <= to)
                }
                None => scope.year_from.is_none() && scope.year_to.is_none(),
            }
        })
        .map(|work| work.id.clone())
        .collect();
    Ok(allowed)
}

fn parse_genre(value: &str) -> Result<Genre, ScopeError> {
    match value.to_ascii_lowercase().as_str() {
        "fachbuch" => Ok(Genre::Fachbuch),
        "sachbuch" => Ok(Genre::Sachbuch),
        "belletristik" => Ok(Genre::Belletristik),
        "sonstiges" => Ok(Genre::Sonstiges),
        _ => Err(ScopeError::InvalidGenre),
    }
}

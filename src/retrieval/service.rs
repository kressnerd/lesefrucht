//! Orchestration: availability, scope, expansion, paths, fusion, outcome.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::domain::{
    AbstentionReason, Citation, Expansion, FailedEdition, IndexingSummary, PassageContext,
    PassageHit, QuestionOutcome, UnavailabilityReason, WorkId,
};
use crate::index::IndexStore;
use crate::indexing::embed::EmbeddingClient;
use crate::indexing::freshness_ok;
use crate::library::{Calibredb, LibraryError};

use super::expand::Expander;
use super::fuse;
use super::scope::{self, Scope, ScopeError};
use super::{lexical, vector};

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchRequest {
    pub claim: String,
    pub language: String,
    #[serde(default)]
    pub scope: Option<Scope>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ReadRequest {
    pub edition_id: String,
    pub passage_index: u32,
    #[serde(default)]
    pub window_chars: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("invalid citation")]
    InvalidCitation,
    #[error("citation integrity check failed: {0}")]
    Integrity(String),
    #[error(transparent)]
    Store(#[from] crate::index::StoreError),
    #[error(transparent)]
    Library(#[from] LibraryError),
    #[error(transparent)]
    Vector(#[from] super::vector::VectorPathError),
}

pub struct SearchService {
    store: Arc<IndexStore>,
    config: Config,
    expander: Expander,
    embedder: EmbeddingClient,
    calibredb: Calibredb,
    generation: Mutex<Option<(Instant, String)>>,
}

impl SearchService {
    pub fn new(store: Arc<IndexStore>, config: Config) -> Self {
        let expander = Expander::load(&config.expansion.synonyms_file);
        let embedder = EmbeddingClient::new(
            config.embedding.base_url.clone(),
            config.embedding.model.clone(),
            config.embedding.dimension,
            config.embedding.batch_size,
            config.embedding.timeout_ms,
        );
        let calibredb = Calibredb::new(
            config.library.calibredb_binary.clone(),
            config.library.url.clone(),
            config.library.timeout_ms,
        );
        Self {
            store,
            config,
            expander,
            embedder,
            calibredb,
            generation: Mutex::new(None),
        }
    }

    pub fn store(&self) -> &Arc<IndexStore> {
        &self.store
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn search(&self, request: SearchRequest) -> Result<QuestionOutcome, ServiceError> {
        if request.claim.trim().is_empty() || request.claim.chars().count() > 2_000 {
            return Err(ServiceError::InvalidArguments(
                "claim must be 1..2000 characters".into(),
            ));
        }
        if request.language.trim().is_empty() {
            return Err(ServiceError::InvalidArguments("language is required".into()));
        }
        if request.limit.is_some_and(|limit| limit < 1) {
            return Err(ServiceError::InvalidArguments(
                "limit must be at least 1".into(),
            ));
        }
        let effective_limit = request
            .limit
            .unwrap_or(self.config.retrieval.limit_default)
            .clamp(1, self.config.retrieval.limit_max);

        let status = self.store.status()?;
        if status.indexed_editions == 0 {
            return Ok(QuestionOutcome::Unavailable {
                reason: UnavailabilityReason::IndexUnavailable,
            });
        }
        let generation = match self.current_generation().await {
            Ok(generation) => generation,
            Err(_) => {
                return Ok(QuestionOutcome::Unavailable {
                    reason: UnavailabilityReason::LibraryUnreachable,
                });
            }
        };
        let generation_changed = status
            .library_generation
            .as_deref()
            .is_some_and(|recorded| recorded != generation);
        if generation_changed || !freshness_ok(&status, &self.config) {
            return Ok(QuestionOutcome::Unavailable {
                reason: UnavailabilityReason::StaleIndex,
            });
        }

        let works = self.store.works()?;
        let (allowed, named_work_ids) = match request.scope.as_ref() {
            Some(scope) => {
                match scope::resolve(scope, &works) {
                    Ok(allowed) => {
                        let named = if scope.work_ids.is_empty() {
                            None
                        } else {
                            Some(scope.work_ids.clone())
                        };
                        (Some(allowed), named)
                    }
                    Err(ScopeError::UnknownWork) => {
                        return Ok(QuestionOutcome::NoEvidence {
                            reason: AbstentionReason::UnknownWork,
                            indexing: None,
                        });
                    }
                    Err(error) => {
                        return Err(ServiceError::InvalidArguments(error.to_string()));
                    }
                }
            }
            None => (None, None),
        };
        let allowed_strings: Option<HashSet<String>> = allowed
            .as_ref()
            .map(|ids| ids.iter().map(|id| id.0.clone()).collect());

        let (expansion, variants) = self.expander.expand(&request.claim);
        let pack = effective_limit.saturating_mul(self.config.retrieval.vector_overfetch.max(1));
        let lexical_hits = lexical::search(&self.store, &variants, allowed_strings.as_ref(), pack)?;
        let query_text = variants.join(" ");
        let vector_hits = vector::search(
            &self.store,
            &self.embedder,
            &query_text,
            allowed_strings.as_ref(),
            pack,
        )
        .await?;
        let fused = fuse::fuse(&lexical_hits, &vector_hits, &self.config.retrieval);
        let selected: Vec<&fuse::FusedHit> = fused.iter().take(effective_limit).collect();

        if selected.is_empty() {
            let failed = self.failed_editions(named_work_ids.as_deref())?;
            return Ok(QuestionOutcome::NoEvidence {
                reason: AbstentionReason::NoEvidence,
                indexing: failed,
            });
        }

        let works_by_id: HashMap<WorkId, _> =
            works.into_iter().map(|work| (work.id.clone(), work)).collect();
        let editions = self.store.editions()?;
        let editions_by_id: HashMap<_, _> = editions
            .into_iter()
            .map(|edition| (edition.id.clone(), edition))
            .collect();
        let uids: Vec<String> = selected.iter().map(|hit| hit.uid.clone()).collect();
        let passages = self.store.passages_by_uids(&uids)?;
        let passages_by_uid: HashMap<_, _> = passages
            .into_iter()
            .map(|passage| (passage.uid.0.clone(), passage))
            .collect();
        let mut texts: HashMap<String, String> = HashMap::new();
        let mut return_hits = Vec::with_capacity(selected.len());
        for hit in &selected {
            let Some(passage) = passages_by_uid.get(&hit.uid) else {
                continue;
            };
            let Some(work) = works_by_id.get(&passage.work_id) else {
                continue;
            };
            let Some(edition) = editions_by_id.get(&passage.edition_id) else {
                continue;
            };
            let Some(content_hash) = edition.content_hash.as_ref() else {
                continue;
            };
            let text = match texts.get(&content_hash.0) {
                Some(text) => text.clone(),
                None => {
                    let text = self.store.text(&content_hash.0)?;
                    texts.insert(content_hash.0.clone(), text.clone());
                    text
                }
            };
            let passage_text = slice_chars(&text, passage.char_start, passage.char_end);
            let digest = hex::encode(Sha256::digest(passage_text.as_bytes()));
            if digest != passage.text_hash {
                return Err(ServiceError::Integrity(format!(
                    "passage {} does not match the stored text",
                    passage.uid.0
                )));
            }
            let matched_expansion = expansion
                .expanded_terms
                .iter()
                .filter(|term| passage_text.to_lowercase().contains(&term.to_lowercase()))
                .cloned()
                .collect();
            return_hits.push(PassageHit {
                citation: Citation {
                    work_id: work.id.clone(),
                    edition_id: edition.id.clone(),
                    format: edition.format.clone(),
                    title: work.title.clone(),
                    authors: work.authors.clone(),
                    language: work.language.clone(),
                    chapter: passage.chapter.clone(),
                    passage_index: passage.passage_index,
                    char_start: passage.char_start,
                    char_end: passage.char_end,
                    viewer_url: self.viewer_url(&work.id),
                },
                score: hit.score,
                expansion: matched_expansion,
                text: passage_text,
            });
        }

        let returned = return_hits.len();
        Ok(QuestionOutcome::CandidatePassages {
            requested: effective_limit,
            returned,
            short: returned < effective_limit,
            expansion,
            passages: return_hits,
        })
    }

    pub async fn read_passage(
        &self,
        request: ReadRequest,
    ) -> Result<PassageContext, ServiceError> {
        let edition_id = crate::domain::EditionId(request.edition_id);
        let passage = self
            .store
            .passage_by_index(&edition_id, request.passage_index)?
            .ok_or(ServiceError::InvalidCitation)?;
        let editions = self.store.editions()?;
        let edition = editions
            .iter()
            .find(|edition| edition.id == edition_id)
            .ok_or(ServiceError::InvalidCitation)?;
        let content_hash = edition
            .content_hash
            .as_ref()
            .ok_or(ServiceError::InvalidCitation)?;
        let text = self.store.text(&content_hash.0)?;
        let cited_text = slice_chars(&text, passage.char_start, passage.char_end);
        let digest = hex::encode(Sha256::digest(cited_text.as_bytes()));
        if digest != passage.text_hash {
            return Err(ServiceError::Integrity(
                "cited span does not match the passage hash".into(),
            ));
        }
        let works = self.store.works()?;
        let work = works
            .into_iter()
            .find(|work| work.id == passage.work_id)
            .ok_or(ServiceError::InvalidCitation)?;
        let (chapter_start, chapter_end) = self
            .store
            .chapter_bounds(&edition_id, passage.chapter.index)?
            .ok_or(ServiceError::InvalidCitation)?;
        let window = request
            .window_chars
            .unwrap_or(0)
            .min(self.config.read.max_window_chars) as u32;
        let requested_start = passage.char_start.saturating_sub(window);
        let requested_end = passage
            .char_end
            .saturating_add(window)
            .min(chapter_end);
        let start = requested_start.max(chapter_start);
        let end = requested_end.max(passage.char_end).min(chapter_end);
        let truncated = start != requested_start || end != passage.char_end.saturating_add(window);
        Ok(PassageContext {
            kind: "passage_context".into(),
            citation: Citation {
                work_id: work.id.clone(),
                edition_id: edition.id.clone(),
                format: edition.format.clone(),
                title: work.title,
                authors: work.authors,
                language: work.language,
                chapter: passage.chapter.clone(),
                passage_index: passage.passage_index,
                char_start: passage.char_start,
                char_end: passage.char_end,
                viewer_url: self.viewer_url(&passage.work_id),
            },
            text: slice_chars(&text, start, end),
            bounds: crate::domain::Bounds {
                start_char: start,
                end_char: end,
                chapter: passage.chapter.clone(),
            },
            cited: crate::domain::Cited {
                start_char: passage.char_start,
                end_char: passage.char_end,
            },
            truncated,
        })
    }

    fn viewer_url(&self, work_id: &WorkId) -> String {
        let library = self
            .config
            .library
            .url
            .split_once('#')
            .map(|(_, library)| library.to_string())
            .filter(|library| !library.is_empty())
            .unwrap_or_else(|| "_".to_string());
        format!("calibre://show-book/{library}/{}", work_id.0)
    }

    fn failed_editions(
        &self,
        work_ids: Option<&[String]>,
    ) -> Result<Option<IndexingSummary>, ServiceError> {
        let failed = self.store.failed_editions()?;
        let failed: Vec<FailedEdition> = failed
            .into_iter()
            .filter(|row| {
                work_ids.is_none_or(|ids| ids.iter().any(|id| *id == row.work_id.0))
            })
            .map(|row| FailedEdition {
                work_id: row.work_id,
                edition_id: row.edition_id,
                format: row.format,
                reason: row.reason,
            })
            .collect();
        if failed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(IndexingSummary { failed }))
        }
    }

    async fn current_generation(&self) -> Result<String, ServiceError> {
        let ttl = Duration::from_millis(self.config.index.generation_cache_ms);
        if !ttl.is_zero() {
            if let Some((at, value)) = self.generation.lock().expect("generation").as_ref() {
                if at.elapsed() < ttl {
                    return Ok(value.clone());
                }
            }
        }
        let books = self.calibredb.list().await?;
        let snapshot = crate::library::discovery(&books, &self.config.edition.format_preference);
        let generation = snapshot.generation;
        *self.generation.lock().expect("generation") = Some((Instant::now(), generation.clone()));
        Ok(generation)
    }
}

pub fn slice_chars(text: &str, start: u32, end: u32) -> String {
    text.chars()
        .skip(start as usize)
        .take(end.saturating_sub(start) as usize)
        .collect()
}

pub fn expansion_summary(expansion: &Expansion) -> String {
    expansion.expanded_terms.join(", ")
}

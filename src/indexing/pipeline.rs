//! discovery -> select -> export -> extract -> chunk -> embed -> store.

use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::domain::{
    Chapter, ContentHash, Edition, FailureReason, IndexState, Passage, PassageUid, Work,
};
use crate::index::{now_ms, IndexStore, PassageRecord};
use crate::library::{Calibredb, LibraryError};

use super::chunk::{Chunk, Chunker};
use super::embed::{EmbedError, EmbeddingClient};
use super::extract::{extract, ExtractError};

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error(transparent)]
    Library(#[from] LibraryError),
    #[error(transparent)]
    Store(#[from] crate::index::StoreError),
    #[error("embedding: {0}")]
    Embed(#[from] EmbedError),
    #[error("temporary directory: {0}")]
    Temp(#[from] std::io::Error),
}

pub struct Indexer {
    store: Arc<IndexStore>,
    config: Config,
    calibredb: Calibredb,
    embedder: EmbeddingClient,
    chunker: Chunker,
}

impl Indexer {
    pub fn new(store: Arc<IndexStore>, config: Config) -> Self {
        let calibredb = Calibredb::new(
            config.library.calibredb_binary.clone(),
            config.library.url.clone(),
            config.library.timeout_ms,
        );
        let embedder = EmbeddingClient::new(
            config.embedding.base_url.clone(),
            config.embedding.model.clone(),
            config.embedding.dimension,
            config.embedding.batch_size,
            config.embedding.timeout_ms,
        );
        let chunker = Chunker::new(config.chunk.max_chars, config.chunk.overlap_chars);
        Self {
            store,
            config,
            calibredb,
            embedder,
            chunker,
        }
    }

    /// One full indexing pass. A failing listing leaves the previous index
    /// intact and reports the library as unreachable at request time.
    pub async fn run(&self) -> Result<(), PipelineError> {
        self.store.repair()?;
        let books = match self.calibredb.list().await {
            Ok(books) => books,
            Err(error) => {
                tracing::warn!(%error, "library listing failed; keeping the previous index");
                return Err(PipelineError::Library(error));
            }
        };
        let snapshot = crate::library::discovery(&books, &self.config.edition.format_preference);
        self.store.replace_library(&snapshot)?;
        let selected: Vec<Edition> = snapshot
            .editions
            .iter()
            .filter(|edition| edition.selected)
            .cloned()
            .collect();

        for edition in selected {
            if let Err(error) = self.index_edition(&snapshot, &edition).await {
                tracing::warn!(edition = %edition.id, %error, "edition indexing failed");
            }
        }

        if self.store.indexed_count()? == 0 {
            self.store.set_meta("index_state", "empty")?;
        } else {
            self.store.set_meta("index_state", "ready")?;
            self.store
                .set_meta("last_success_at", &now_ms().to_string())?;
            self.store
                .set_meta("library_generation", &snapshot.generation)?;
        }
        Ok(())
    }

    async fn index_edition(
        &self,
        snapshot: &crate::library::LibrarySnapshot,
        edition: &Edition,
    ) -> Result<(), PipelineError> {
        let checked_at = now_ms().to_string();
        if !self
            .config
            .extraction
            .supported
            .iter()
            .any(|format| format.eq_ignore_ascii_case(&edition.format))
        {
            self.store.mark_failed(
                &edition.id,
                FailureReason::UnsupportedFormat,
                None,
                Some(&checked_at),
            )?;
            return Ok(());
        }
        let work_id = edition
            .work_id
            .numeric()
            .ok_or_else(|| LibraryError::InvalidOutput(format!("work id {}", edition.work_id.0)))?;
        let temp = tempfile::tempdir()?;
        let path = self.calibredb.export(work_id, &edition.format, temp.path()).await?;
        let bytes = tokio::fs::read(&path).await?;
        let content_hash = ContentHash(hex::encode(Sha256::digest(&bytes)));

        let state = self.store.edition_state(&edition.id)?;
        if state == Some(IndexState::Indexed)
            && edition
                .content_hash
                .as_ref()
                .is_some_and(|recorded| recorded.0 == content_hash.0)
        {
            self.store.mark_checked(&edition.id, &checked_at)?;
            return Ok(());
        }

        self.store.begin_indexing(&edition.id)?;
        let chapters = match extract(&edition.format, &bytes) {
            Ok(chapters) => chapters,
            Err(ExtractError::Unsupported) => {
                self.store.mark_failed(
                    &edition.id,
                    FailureReason::UnsupportedFormat,
                    Some(&content_hash.0),
                    Some(&checked_at),
                )?;
                return Ok(());
            }
            Err(ExtractError::EmptyText) => {
                self.store.mark_failed(
                    &edition.id,
                    FailureReason::EmptyText,
                    Some(&content_hash.0),
                    Some(&checked_at),
                )?;
                return Ok(());
            }
            Err(ExtractError::Failed(error)) => {
                tracing::warn!(edition = %edition.id, %error, "extraction failed");
                self.store.mark_failed(
                    &edition.id,
                    FailureReason::ExtractionFailed,
                    Some(&content_hash.0),
                    Some(&checked_at),
                )?;
                return Ok(());
            }
        };

        let (text, chapter_ranges) = normalize_text(&chapters);
        let chunks = self.chunk_chapters(&chapters, &chapter_ranges);
        let texts: Vec<String> = chunks.iter().map(|chunk| chunk.text.clone()).collect();
        let vectors = match self.embedder.embed(&texts).await {
            Ok(vectors) => vectors,
            Err(error) => {
                tracing::warn!(edition = %edition.id, %error, "embedding failed");
                self.store.mark_failed(
                    &edition.id,
                    FailureReason::EmbeddingFailed,
                    Some(&content_hash.0),
                    Some(&checked_at),
                )?;
                return Ok(());
            }
        };
        let records = passage_records(
            &content_hash,
            &edition.work_id,
            &edition.id,
            &chunks,
        );
        let work = snapshot
            .works
            .iter()
            .find(|work| work.id == edition.work_id)
            .cloned()
            .unwrap_or_else(|| placeholder_work(&edition.work_id));
        self.store.commit_edition(
            &work,
            edition,
            &chapters,
            &chapter_ranges,
            &records,
            &vectors,
            &text,
            &content_hash,
            &checked_at,
            &snapshot.generation,
        )?;
        Ok(())
    }

    fn chunk_chapters(
        &self,
        chapters: &[Chapter],
        ranges: &[(u32, u32)],
    ) -> Vec<Chunk> {
        let mut next_index = 0;
        let mut chunks = Vec::new();
        for (chapter, (start, _)) in chapters.iter().zip(ranges.iter()) {
            chunks.extend(self.chunker.chunk(chapter, *start, &mut next_index));
        }
        chunks
    }
}

fn normalize_text(chapters: &[Chapter]) -> (String, Vec<(u32, u32)>) {
    let mut text = String::new();
    let mut ranges = Vec::with_capacity(chapters.len());
    let mut offset = 0u32;
    for chapter in chapters {
        let start = offset;
        text.push_str(&chapter.text);
        offset += chapter.text.chars().count() as u32;
        ranges.push((start, offset));
    }
    (text, ranges)
}

fn passage_records(
    content_hash: &ContentHash,
    work_id: &crate::domain::WorkId,
    edition_id: &crate::domain::EditionId,
    chunks: &[Chunk],
) -> Vec<PassageRecord> {
    chunks
        .iter()
        .map(|chunk| {
            let mut hasher = Sha256::new();
            hasher.update(content_hash.0.as_bytes());
            hasher.update(b"|");
            hasher.update(chunk.chapter.index.to_be_bytes());
            hasher.update(b"|");
            hasher.update(chunk.passage_index.to_be_bytes());
            hasher.update(b"|");
            hasher.update(chunk.char_start.to_be_bytes());
            hasher.update(b"|");
            hasher.update(chunk.char_end.to_be_bytes());
            PassageRecord {
                passage: Passage {
                    uid: PassageUid(hex::encode(hasher.finalize())),
                    work_id: work_id.clone(),
                    edition_id: edition_id.clone(),
                    passage_index: chunk.passage_index,
                    chapter: chunk.chapter.clone(),
                    char_start: chunk.char_start,
                    char_end: chunk.char_end,
                    text_hash: chunk.text_hash.clone(),
                },
                text: chunk.text.clone(),
            }
        })
        .collect()
}

fn placeholder_work(work_id: &crate::domain::WorkId) -> Work {
    Work {
        id: work_id.clone(),
        title: String::new(),
        authors: Vec::new(),
        language: "und".into(),
        genre: None,
        pubdate: None,
    }
}

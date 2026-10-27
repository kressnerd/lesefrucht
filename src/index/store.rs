//! The only writer: SQLite, tantivy, usearch and normalized text.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::config::Config;
use crate::domain::{
    Chapter, ContentHash, Edition, EditionId, FailureReason, IndexState, Passage, PassageUid, Work,
    WorkId,
};
use crate::library::LibrarySnapshot;

use super::lexical_index::{LexicalDoc, LexicalError, LexicalIndex};
use super::sqlite::{Db, DbError, FailedRow};
use super::text_store::TextStore;
use super::vector_index::{VectorError, VectorIndex};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Lexical(#[from] LexicalError),
    #[error(transparent)]
    Vector(#[from] VectorError),
    #[error("text store: {0}")]
    Text(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct PassageRecord {
    pub passage: Passage,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct StoreStatus {
    pub indexed_editions: usize,
    pub failed_editions: Vec<FailedRow>,
    pub last_success_at: Option<u64>,
    pub library_generation: Option<String>,
    pub state: String,
}

pub struct IndexStore {
    dir: PathBuf,
    db: Mutex<Db>,
    lexical: LexicalIndex,
    vectors: VectorIndex,
    text: TextStore,
    dimension: usize,
}

impl IndexStore {
    pub fn open(config: &Config) -> Result<Self, StoreError> {
        let dir = config.index.dir.clone();
        std::fs::create_dir_all(&dir)?;
        let db = Db::open(&dir.join("index.sqlite"))?;
        let lexical = LexicalIndex::open(&dir.join("lexical"))?;
        let vectors = VectorIndex::open(&dir, config.embedding.dimension)?;
        let text = TextStore::new(dir.join("text"));
        Ok(Self {
            dir,
            db: Mutex::new(db),
            lexical,
            vectors,
            text,
            dimension: config.embedding.dimension,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Mirror the latest listing into `works`/`editions`.
    pub fn replace_library(&self, snapshot: &LibrarySnapshot) -> Result<(), StoreError> {
        let db = self.db.lock().expect("store db");
        db.replace_works(&snapshot.works, &snapshot.editions)?;
        Ok(())
    }

    pub fn edition_state(&self, edition_id: &EditionId) -> Result<Option<IndexState>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.edition_state(edition_id)?)
    }

    pub fn has_content(&self, edition_id: &EditionId) -> Result<bool, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.count_where(&format!(
            "SELECT COUNT(*) FROM passages WHERE edition_id = '{}'",
            edition_id.0.replace('\'', "''")
        ))? > 0)
    }

    pub fn begin_indexing(&self, edition_id: &EditionId) -> Result<(), StoreError> {
        let db = self.db.lock().expect("store db");
        db.delete_edition_content(edition_id)?;
        db.set_edition_state(edition_id, &IndexState::Indexing, None, None, None)?;
        drop(db);
        self.lexical.delete_edition(&edition_id.0)?;
        self.vectors.remove_edition(&edition_id.0)?;
        Ok(())
    }

    pub fn mark_failed(
        &self,
        edition_id: &EditionId,
        reason: FailureReason,
        content_hash: Option<&str>,
        checked_at: Option<&str>,
    ) -> Result<(), StoreError> {
        let db = self.db.lock().expect("store db");
        db.set_edition_state(
            edition_id,
            &IndexState::Failed(reason),
            content_hash,
            Some(reason),
            checked_at,
        )?;
        Ok(())
    }

    pub fn mark_checked(&self, edition_id: &EditionId, checked_at: &str) -> Result<(), StoreError> {
        let db = self.db.lock().expect("store db");
        db.set_edition_state(edition_id, &IndexState::Indexed, None, None, Some(checked_at))?;
        Ok(())
    }

    pub fn commit_edition(
        &self,
        work: &Work,
        edition: &Edition,
        chapters: &[Chapter],
        chapter_ranges: &[(u32, u32)],
        passages: &[PassageRecord],
        vectors: &[Vec<f32>],
        text: &str,
        content_hash: &ContentHash,
        checked_at: &str,
        generation: &str,
    ) -> Result<(), StoreError> {
        if vectors.len() != passages.len() {
            return Err(StoreError::Vector(VectorError::Usearch(
                "vector count does not match passage count".into(),
            )));
        }
        self.text.write(&content_hash.0, text)?;
        {
            let db = self.db.lock().expect("store db");
            db.delete_edition_content(&edition.id)?;
            for (chapter, (start, end)) in chapters.iter().zip(chapter_ranges.iter()) {
                db.insert_chapter(
                    &edition.id,
                    chapter.index,
                    chapter.title.as_deref(),
                    *start,
                    *end,
                )?;
            }
            for (index, record) in passages.iter().enumerate() {
                db.insert_passage(&record.passage, index as u64)?;
            }
            db.set_edition_state(
                &edition.id,
                &IndexState::Indexing,
                Some(&content_hash.0),
                None,
                Some(checked_at),
            )?;
        }
        self.lexical.delete_edition(&edition.id.0)?;
        for record in passages {
            self.lexical.add(&LexicalDoc {
                uid: record.passage.uid.0.clone(),
                work_id: record.passage.work_id.0.clone(),
                edition_id: record.passage.edition_id.0.clone(),
                passage_index: record.passage.passage_index,
                text: record.text.clone(),
            })?;
        }
        self.lexical.commit()?;
        self.vectors.remove_edition(&edition.id.0)?;
        for (record, vector) in passages.iter().zip(vectors.iter()) {
            self.vectors.add(
                &record.passage.uid.0,
                &record.passage.work_id.0,
                &record.passage.edition_id.0,
                vector,
            )?;
        }
        self.vectors.save()?;
        {
            let db = self.db.lock().expect("store db");
            db.set_edition_state(
                &edition.id,
                &IndexState::Indexed,
                Some(&content_hash.0),
                None,
                Some(checked_at),
            )?;
            db.set_meta("last_success_at", &now_ms().to_string())?;
            db.set_meta("library_generation", generation)?;
            db.set_meta("index_state", "ready")?;
        }
        let _ = work;
        Ok(())
    }

    pub fn works(&self) -> Result<Vec<Work>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.works()?)
    }

    pub fn editions(&self) -> Result<Vec<Edition>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.editions()?)
    }

    pub fn selected_editions(&self) -> Result<Vec<Edition>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.selected_editions()?)
    }

    pub fn passages_for_edition(
        &self,
        edition_id: &EditionId,
    ) -> Result<Vec<Passage>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.passages_for_edition(edition_id)?)
    }

    pub fn passage_by_index(
        &self,
        edition_id: &EditionId,
        passage_index: u32,
    ) -> Result<Option<Passage>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.passage_by_index(edition_id, passage_index)?)
    }

    pub fn passages_by_uids(&self, uids: &[String]) -> Result<Vec<Passage>, StoreError> {
        let db = self.db.lock().expect("store db");
        let mut passages = Vec::with_capacity(uids.len());
        for uid in uids {
            if let Some(passage) = db.passage_by_uid(&PassageUid(uid.clone()))? {
                passages.push(passage);
            }
        }
        Ok(passages)
    }

    pub fn work(&self, work_id: &WorkId) -> Result<Option<Work>, StoreError> {
        Ok(self.works()?.into_iter().find(|work| &work.id == work_id))
    }

    pub fn chapter_bounds(
        &self,
        edition_id: &EditionId,
        chapter_index: u16,
    ) -> Result<Option<(u32, u32)>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.chapter_bounds(edition_id, chapter_index)?)
    }

    pub fn text(&self, content_hash: &str) -> Result<String, StoreError> {
        Ok(self.text.read(content_hash)?)
    }

    pub fn text_path(&self, content_hash: &str) -> PathBuf {
        self.text.path_for(content_hash)
    }

    pub fn meta(&self, key: &str) -> Result<Option<String>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.meta(key)?)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let db = self.db.lock().expect("store db");
        db.set_meta(key, value)?;
        Ok(())
    }

    pub fn status(&self) -> Result<StoreStatus, StoreError> {
        let db = self.db.lock().expect("store db");
        let indexed_editions = db.count_where(
            "SELECT COUNT(*) FROM editions WHERE index_state = 'indexed' AND selected = 1",
        )? as usize;
        let failed_editions = db.failed_editions()?;
        let last_success_at = db
            .meta("last_success_at")?
            .and_then(|value| value.parse().ok());
        let library_generation = db.meta("library_generation")?;
        let state = if indexed_editions == 0 {
            if db.count_where("SELECT COUNT(*) FROM editions WHERE index_state = 'indexing'")? > 0 {
                "indexing".into()
            } else {
                "empty".into()
            }
        } else {
            db.meta("index_state")?
                .unwrap_or_else(|| "ready".into())
        };
        Ok(StoreStatus {
            indexed_editions,
            failed_editions,
            last_success_at,
            library_generation,
            state,
        })
    }

    pub fn indexed_count(&self) -> Result<usize, StoreError> {
        Ok(self.status()?.indexed_editions)
    }

    pub fn failed_editions(&self) -> Result<Vec<FailedRow>, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.failed_editions()?)
    }

    pub fn glossary_count(&self) -> Result<usize, StoreError> {
        let db = self.db.lock().expect("store db");
        Ok(db.count("glossary_terms")? as usize)
    }

    pub fn lexical_search(
        &self,
        tokens: &[String],
        allowed: Option<&HashSet<String>>,
        limit: usize,
    ) -> Result<Vec<super::lexical_index::LexicalHit>, StoreError> {
        Ok(self.lexical.search(tokens, allowed, limit)?)
    }

    pub fn vector_search(
        &self,
        query: &[f32],
        allowed: Option<&HashSet<String>>,
        limit: usize,
    ) -> Result<Vec<super::vector_index::VectorHit>, StoreError> {
        Ok(self.vectors.search(query, allowed, limit)?)
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Startup consistency pass: invalidate editions whose vector map or text
    /// is missing after a crash between the three stores.
    pub fn repair(&self) -> Result<(), StoreError> {
        let editions = self.selected_editions()?;
        for edition in editions {
            if edition.index_state != IndexState::Indexed {
                continue;
            }
            let Some(content_hash) = edition.content_hash.clone() else {
                continue;
            };
            if !self.text.path_for(&content_hash.0).exists() {
                let db = self.db.lock().expect("store db");
                db.set_edition_state(
                    &edition.id,
                    &IndexState::Invalidated,
                    None,
                    None,
                    None,
                )?;
            }
        }
        Ok(())
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

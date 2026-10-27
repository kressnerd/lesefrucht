//! SQLite: works mirror, editions, chapters, passages, vectors map, index meta.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::{
    ChapterRef, Edition, EditionId, FailureReason, Genre, IndexState, Passage, PassageUid, Work,
    WorkId,
};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct Db {
    pub(crate) conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS works (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                authors TEXT NOT NULL,
                language TEXT NOT NULL,
                genre TEXT,
                pubdate TEXT
            );
            CREATE TABLE IF NOT EXISTS editions (
                id TEXT PRIMARY KEY,
                work_id TEXT NOT NULL,
                format TEXT NOT NULL,
                content_hash TEXT,
                selected INTEGER NOT NULL,
                index_state TEXT NOT NULL,
                failure_reason TEXT,
                last_checked_at TEXT
            );
            CREATE INDEX IF NOT EXISTS editions_work ON editions(work_id);
            CREATE TABLE IF NOT EXISTS chapters (
                edition_id TEXT NOT NULL,
                chapter_index INTEGER NOT NULL,
                title TEXT,
                char_start INTEGER NOT NULL,
                char_end INTEGER NOT NULL,
                PRIMARY KEY (edition_id, chapter_index)
            );
            CREATE TABLE IF NOT EXISTS passages (
                uid TEXT PRIMARY KEY,
                work_id TEXT NOT NULL,
                edition_id TEXT NOT NULL,
                passage_index INTEGER NOT NULL,
                chapter_index INTEGER NOT NULL,
                chapter_title TEXT,
                char_start INTEGER NOT NULL,
                char_end INTEGER NOT NULL,
                text_hash TEXT NOT NULL,
                UNIQUE (edition_id, passage_index)
            );
            CREATE INDEX IF NOT EXISTS passages_work ON passages(work_id);
            CREATE INDEX IF NOT EXISTS passages_edition ON passages(edition_id);
            CREATE TABLE IF NOT EXISTS passage_vectors (
                key INTEGER PRIMARY KEY,
                uid TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS glossary_terms (
                term TEXT NOT NULL,
                language TEXT NOT NULL,
                expansion TEXT NOT NULL,
                PRIMARY KEY (term, language)
            );
            CREATE TABLE IF NOT EXISTS index_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
        self.set_meta_if_absent("index_state", "empty")?;
        Ok(())
    }

    pub fn replace_works(&self, works: &[Work], editions: &[Edition]) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM works", [])?;
        for work in works {
            tx.execute(
                "INSERT INTO works (id, title, authors, language, genre, pubdate) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    work.id.0,
                    work.title,
                    serde_json::to_string(&work.authors)?,
                    work.language,
                    work.genre.map(|genre| genre.as_str()),
                    work.pubdate,
                ],
            )?;
        }
        for edition in editions {
            let existing: Option<(Option<String>, Option<String>, Option<String>, Option<String>)> = tx
                .query_row(
                    "SELECT content_hash, index_state, failure_reason, last_checked_at FROM editions WHERE id = ?1",
                    params![edition.id.0],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
                .optional()?;
            let (state, reason, checked) = match existing {
                Some((_, state, reason, checked)) => (
                    state.unwrap_or_else(|| "not_indexed".into()),
                    reason,
                    checked,
                ),
                None => ("not_indexed".into(), None, None),
            };
            tx.execute(
                "INSERT INTO editions (id, work_id, format, content_hash, selected, index_state, failure_reason, last_checked_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET work_id=excluded.work_id, format=excluded.format, content_hash=excluded.content_hash, selected=excluded.selected, index_state=excluded.index_state, failure_reason=excluded.failure_reason, last_checked_at=excluded.last_checked_at",
                params![
                    edition.id.0,
                    edition.work_id.0,
                    edition.format,
                    edition.content_hash.as_ref().map(|hash| hash.0.clone()),
                    i64::from(edition.selected),
                    state,
                    reason,
                    checked,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_edition_content(&self, edition_id: &EditionId) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM passages WHERE edition_id = ?1",
            params![edition_id.0],
        )?;
        self.conn.execute(
            "DELETE FROM chapters WHERE edition_id = ?1",
            params![edition_id.0],
        )?;
        Ok(())
    }

    pub fn set_edition_state(
        &self,
        edition_id: &EditionId,
        state: &IndexState,
        content_hash: Option<&str>,
        failure: Option<FailureReason>,
        checked_at: Option<&str>,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "UPDATE editions SET index_state = ?2, content_hash = COALESCE(?3, content_hash), failure_reason = ?4, last_checked_at = ?5 WHERE id = ?1",
            params![
                edition_id.0,
                state_name(state),
                content_hash,
                failure.map(|reason| reason.as_str()),
                checked_at,
            ],
        )?;
        Ok(())
    }

    pub fn insert_chapter(
        &self,
        edition_id: &EditionId,
        index: u16,
        title: Option<&str>,
        char_start: u32,
        char_end: u32,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO chapters (edition_id, chapter_index, title, char_start, char_end) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![edition_id.0, index, title, char_start, char_end],
        )?;
        Ok(())
    }

    pub fn insert_passage(
        &self,
        passage: &Passage,
        key: u64,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO passages (uid, work_id, edition_id, passage_index, chapter_index, chapter_title, char_start, char_end, text_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                passage.uid.0,
                passage.work_id.0,
                passage.edition_id.0,
                passage.passage_index,
                passage.chapter.index,
                passage.chapter.title,
                passage.char_start,
                passage.char_end,
                passage.text_hash,
            ],
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO passage_vectors (key, uid) VALUES (?1, ?2)",
            params![key as i64, passage.uid.0],
        )?;
        Ok(())
    }

    pub fn works(&self) -> Result<Vec<Work>, DbError> {
        let mut statement = self.conn.prepare(
            "SELECT id, title, authors, language, genre, pubdate FROM works ORDER BY CAST(id AS INTEGER), id",
        )?;
        let rows = statement.query_map([], |row| {
            let authors: String = row.get(2)?;
            let genre: Option<String> = row.get(4)?;
            Ok(Work {
                id: WorkId(row.get(0)?),
                title: row.get(1)?,
                authors: serde_json::from_str(&authors).unwrap_or_default(),
                language: row.get(3)?,
                genre: genre.as_deref().and_then(genre_from_str),
                pubdate: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn works_from_metadata(&self) -> Result<Vec<Work>, DbError> {
        self.works()
    }

    pub fn edition_state(
        &self,
        edition_id: &EditionId,
    ) -> Result<Option<IndexState>, DbError> {
        let raw: Option<(String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT index_state, failure_reason FROM editions WHERE id = ?1",
                params![edition_id.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        Ok(raw.map(|(state, reason)| state_from_str(&state, reason.as_deref())))
    }

    pub fn editions(&self) -> Result<Vec<Edition>, DbError> {
        let mut statement = self.conn.prepare(
            "SELECT id, work_id, format, content_hash, selected, index_state, failure_reason, last_checked_at FROM editions ORDER BY work_id, format",
        )?;
        let rows = statement.query_map([], |row| {
            let state: String = row.get(5)?;
            let reason: Option<String> = row.get(6)?;
            Ok(Edition {
                id: EditionId(row.get(0)?),
                work_id: WorkId(row.get(1)?),
                format: row.get(2)?,
                content_hash: row
                    .get::<_, Option<String>>(3)?
                    .map(crate::domain::ContentHash),
                selected: row.get::<_, i64>(4)? != 0,
                index_state: state_from_str(&state, reason.as_deref()),
                last_checked_at: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn selected_editions(&self) -> Result<Vec<Edition>, DbError> {
        Ok(self.editions()?.into_iter().filter(|edition| edition.selected).collect())
    }

    pub fn passages_for_edition(
        &self,
        edition_id: &EditionId,
    ) -> Result<Vec<Passage>, DbError> {
        let mut statement = self.conn.prepare(
            "SELECT uid, work_id, edition_id, passage_index, chapter_index, chapter_title, char_start, char_end, text_hash FROM passages WHERE edition_id = ?1 ORDER BY passage_index",
        )?;
        let rows = statement.query_map(params![edition_id.0], |row| {
            Ok(Passage {
                uid: PassageUid(row.get(0)?),
                work_id: WorkId(row.get(1)?),
                edition_id: EditionId(row.get(2)?),
                passage_index: row.get(3)?,
                chapter: ChapterRef {
                    index: row.get(4)?,
                    title: row.get(5)?,
                },
                char_start: row.get(6)?,
                char_end: row.get(7)?,
                text_hash: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn passage_by_uid(&self, uid: &PassageUid) -> Result<Option<Passage>, DbError> {
        self.conn
            .query_row(
                "SELECT uid, work_id, edition_id, passage_index, chapter_index, chapter_title, char_start, char_end, text_hash FROM passages WHERE uid = ?1",
                params![uid.0],
                |row| {
                    Ok(Passage {
                        uid: PassageUid(row.get(0)?),
                        work_id: WorkId(row.get(1)?),
                        edition_id: EditionId(row.get(2)?),
                        passage_index: row.get(3)?,
                        chapter: ChapterRef {
                            index: row.get(4)?,
                            title: row.get(5)?,
                        },
                        char_start: row.get(6)?,
                        char_end: row.get(7)?,
                        text_hash: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn passage_by_index(
        &self,
        edition_id: &EditionId,
        passage_index: u32,
    ) -> Result<Option<Passage>, DbError> {
        let uid: Option<String> = self
            .conn
            .query_row(
                "SELECT uid FROM passages WHERE edition_id = ?1 AND passage_index = ?2",
                params![edition_id.0, passage_index],
                |row| row.get(0),
            )
            .optional()?;
        match uid {
            Some(uid) => self.passage_by_uid(&PassageUid(uid)),
            None => Ok(None),
        }
    }

    pub fn chapter_bounds(
        &self,
        edition_id: &EditionId,
        chapter_index: u16,
    ) -> Result<Option<(u32, u32)>, DbError> {
        self.conn
            .query_row(
                "SELECT char_start, char_end FROM chapters WHERE edition_id = ?1 AND chapter_index = ?2",
                params![edition_id.0, chapter_index],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO index_meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn meta(&self, key: &str) -> Result<Option<String>, DbError> {
        self.conn
            .query_row(
                "SELECT value FROM index_meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    fn set_meta_if_absent(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO index_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn count(&self, table: &str) -> Result<i64, DbError> {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        Ok(self.conn.query_row(&sql, [], |row| row.get(0))?)
    }

    pub fn count_where(&self, sql: &str) -> Result<i64, DbError> {
        Ok(self.conn.query_row(sql, [], |row| row.get(0))?)
    }

    pub fn failed_editions(&self) -> Result<Vec<FailedRow>, DbError> {
        let mut statement = self.conn.prepare(
            "SELECT id, work_id, format, failure_reason FROM editions WHERE index_state = 'failed' ORDER BY work_id, format",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(FailedRow {
                edition_id: EditionId(row.get(0)?),
                work_id: WorkId(row.get(1)?),
                format: row.get(2)?,
                reason: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct FailedRow {
    pub edition_id: EditionId,
    pub work_id: WorkId,
    pub format: String,
    pub reason: String,
}

pub fn state_name(state: &IndexState) -> String {
    match state {
        IndexState::NotIndexed => "not_indexed".into(),
        IndexState::Indexing => "indexing".into(),
        IndexState::Indexed => "indexed".into(),
        IndexState::Failed(_) => "failed".into(),
        IndexState::Invalidated => "invalidated".into(),
    }
}

pub fn state_from_str(state: &str, reason: Option<&str>) -> IndexState {
    match state {
        "indexed" => IndexState::Indexed,
        "indexing" => IndexState::Indexing,
        "invalidated" => IndexState::Invalidated,
        "failed" => IndexState::Failed(match reason {
            Some("unsupported_format") => FailureReason::UnsupportedFormat,
            Some("extraction_failed") => FailureReason::ExtractionFailed,
            Some("embedding_failed") => FailureReason::EmbeddingFailed,
            Some("source_unreachable") => FailureReason::SourceUnreachable,
            _ => FailureReason::EmptyText,
        }),
        _ => IndexState::NotIndexed,
    }
}

fn genre_from_str(value: &str) -> Option<Genre> {
    match value {
        "fachbuch" => Some(Genre::Fachbuch),
        "sachbuch" => Some(Genre::Sachbuch),
        "belletristik" => Some(Genre::Belletristik),
        "sonstiges" => Some(Genre::Sonstiges),
        _ => None,
    }
}

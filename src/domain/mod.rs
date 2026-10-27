//! The domain vocabulary: works, editions, chapters, passages, citations.

pub mod ids;
pub mod outcome;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use ids::{ContentHash, EditionId, PassageUid, WorkId};
pub use outcome::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Genre {
    Fachbuch,
    Sachbuch,
    Belletristik,
    Sonstiges,
}

impl Genre {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag.strip_prefix("genre.")?.to_ascii_lowercase().as_str() {
            "fachbuch" => Some(Self::Fachbuch),
            "sachbuch" => Some(Self::Sachbuch),
            "belletristik" => Some(Self::Belletristik),
            "sonstiges" => Some(Self::Sonstiges),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fachbuch => "fachbuch",
            Self::Sachbuch => "sachbuch",
            Self::Belletristik => "belletristik",
            Self::Sonstiges => "sonstiges",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Work {
    pub id: WorkId,
    pub title: String,
    pub authors: Vec<String>,
    pub language: String,
    pub genre: Option<Genre>,
    pub pubdate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum IndexState {
    NotIndexed,
    Indexing,
    Indexed,
    Failed(FailureReason),
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    UnsupportedFormat,
    ExtractionFailed,
    EmbeddingFailed,
    SourceUnreachable,
    EmptyText,
}

impl FailureReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedFormat => "unsupported_format",
            Self::ExtractionFailed => "extraction_failed",
            Self::EmbeddingFailed => "embedding_failed",
            Self::SourceUnreachable => "source_unreachable",
            Self::EmptyText => "empty_text",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Edition {
    pub id: EditionId,
    pub work_id: WorkId,
    pub format: String,
    pub content_hash: Option<ContentHash>,
    pub selected: bool,
    pub index_state: IndexState,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Chapter {
    pub index: u16,
    pub title: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChapterRef {
    pub index: u16,
    pub title: Option<String>,
}

impl From<&Chapter> for ChapterRef {
    fn from(chapter: &Chapter) -> Self {
        Self {
            index: chapter.index,
            title: chapter.title.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Passage {
    pub uid: PassageUid,
    pub work_id: WorkId,
    pub edition_id: EditionId,
    pub passage_index: u32,
    pub chapter: ChapterRef,
    pub char_start: u32,
    pub char_end: u32,
    pub text_hash: String,
}

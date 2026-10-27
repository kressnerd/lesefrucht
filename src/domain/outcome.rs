//! The exactly-one `Question` outcome.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ChapterRef, EditionId, WorkId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AbstentionReason {
    NoEvidence,
    UnknownWork,
}

impl AbstentionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NoEvidence => "no_evidence",
            Self::UnknownWork => "unknown_work",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnavailabilityReason {
    IndexUnavailable,
    StaleIndex,
    LibraryUnreachable,
}

impl UnavailabilityReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndexUnavailable => "index_unavailable",
            Self::StaleIndex => "stale_index",
            Self::LibraryUnreachable => "library_unreachable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Citation {
    pub work_id: WorkId,
    pub edition_id: EditionId,
    pub format: String,
    pub title: String,
    pub authors: Vec<String>,
    pub language: String,
    pub chapter: ChapterRef,
    pub passage_index: u32,
    pub char_start: u32,
    pub char_end: u32,
    pub viewer_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PassageHit {
    pub citation: Citation,
    pub score: f32,
    pub expansion: Vec<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Expansion {
    pub claim_tokens: Vec<String>,
    pub expanded_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FailedEdition {
    pub work_id: WorkId,
    pub edition_id: EditionId,
    pub format: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IndexingSummary {
    pub failed: Vec<FailedEdition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuestionOutcome {
    CandidatePassages {
        requested: usize,
        returned: usize,
        short: bool,
        expansion: Expansion,
        passages: Vec<PassageHit>,
    },
    NoEvidence {
        reason: AbstentionReason,
        #[serde(skip_serializing_if = "Option::is_none")]
        indexing: Option<IndexingSummary>,
    },
    Unavailable {
        reason: UnavailabilityReason,
    },
}

impl QuestionOutcome {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CandidatePassages { .. } => "candidate_passages",
            Self::NoEvidence { .. } => "no_evidence",
            Self::Unavailable { .. } => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Bounds {
    pub start_char: u32,
    pub end_char: u32,
    pub chapter: ChapterRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Cited {
    pub start_char: u32,
    pub end_char: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PassageContext {
    pub kind: String,
    pub citation: Citation,
    pub text: String,
    pub bounds: Bounds,
    pub cited: Cited,
    pub truncated: bool,
}

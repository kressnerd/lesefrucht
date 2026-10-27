//! Evidence threshold, reciprocal rank fusion and deterministic ordering.

use std::collections::HashMap;

use crate::config::RetrievalConfig;
use crate::domain::EditionId;
use crate::domain::WorkId;
use crate::index::lexical_index::LexicalHit;
use crate::index::vector_index::VectorHit;

#[derive(Debug, Clone)]
pub struct FusedHit {
    pub uid: String,
    pub work_id: WorkId,
    pub edition_id: EditionId,
    pub passage_index: u32,
    pub score: f32,
}

#[derive(Debug, Default, Clone)]
struct Candidate {
    uid: String,
    work_id: String,
    edition_id: String,
    passage_index: u32,
    bm25: Option<f32>,
    cosine: Option<f32>,
    lexical_rank: Option<usize>,
    vector_rank: Option<usize>,
}

pub fn fuse(
    lexical: &[LexicalHit],
    vector: &[VectorHit],
    config: &RetrievalConfig,
) -> Vec<FusedHit> {
    let mut candidates: HashMap<String, Candidate> = HashMap::new();
    for (rank, hit) in lexical.iter().enumerate() {
        let entry = candidates.entry(hit.uid.clone()).or_insert_with(|| Candidate {
            uid: hit.uid.clone(),
            work_id: hit.work_id.clone(),
            edition_id: hit.edition_id.clone(),
            passage_index: hit.passage_index,
            ..Candidate::default()
        });
        entry.bm25 = Some(hit.score);
        entry.lexical_rank = Some(rank + 1);
    }
    for (rank, hit) in vector.iter().enumerate() {
        let entry = candidates.entry(hit.uid.clone()).or_insert_with(|| Candidate {
            uid: hit.uid.clone(),
            work_id: hit.work_id.clone(),
            edition_id: hit.edition_id.clone(),
            ..Candidate::default()
        });
        entry.cosine = Some(hit.similarity);
        entry.vector_rank = Some(rank + 1);
    }

    let mut survivors: Vec<FusedHit> = candidates
        .into_values()
        .filter(|candidate| {
            candidate.bm25.is_some_and(|score| score >= config.lexical_min_score)
                || candidate
                    .cosine
                    .is_some_and(|score| score >= config.vector_min_cosine)
        })
        .map(|candidate| {
            let mut score = 0.0f32;
            if let Some(rank) = candidate.lexical_rank {
                score += 1.0 / (config.rrf_k + rank as f32);
            }
            if let Some(rank) = candidate.vector_rank {
                score += 1.0 / (config.rrf_k + rank as f32);
            }
            FusedHit {
                uid: candidate.uid,
                work_id: WorkId(candidate.work_id),
                edition_id: EditionId(candidate.edition_id),
                passage_index: candidate.passage_index,
                score,
            }
        })
        .collect();

    survivors.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.work_id.cmp(&right.work_id))
            .then_with(|| left.passage_index.cmp(&right.passage_index))
    });
    survivors
}

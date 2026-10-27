//! Non-`@e2e` step definitions for `passage-retrieval`.

use std::collections::HashMap;

use cucumber::{given, then, when};
use lesefrucht::domain::{Citation, Genre, QuestionOutcome, UnavailabilityReason, Work, WorkId};
use lesefrucht::retrieval::scope::Scope;

use super::support::world::World;

const CLAIM: &str = "Vertrauen in Organisationen";
const RARE: &str = "Sonderbegriff";
const ABSENT: &str = "Quantenverschraenkung";

fn works(world: &World) -> HashMap<WorkId, Work> {
    world
        .store()
        .works()
        .expect("works")
        .into_iter()
        .map(|work| (work.id.clone(), work))
        .collect()
}

fn candidate(world: &World) -> (usize, usize, bool, Vec<Citation>) {
    match world.outcome.as_ref().unwrap_or_else(|| panic!("no outcome (error: {:?})", world.error)) {
        QuestionOutcome::CandidatePassages {
            requested,
            returned,
            short,
            passages,
            ..
        } => (
            *requested,
            *returned,
            *short,
            passages.iter().map(|hit| hit.citation.clone()).collect(),
        ),
        other => panic!("expected candidate passages, got {other:?}"),
    }
}

fn assert_abstention(world: &World, reason: &str) {
    match world.outcome.as_ref().unwrap_or_else(|| panic!("no outcome (error: {:?})", world.error)) {
        QuestionOutcome::NoEvidence { reason: actual, .. } => {
            assert_eq!(actual.as_str(), reason, "abstention reason");
        }
        other => panic!("expected an abstention, got {other:?}"),
    }
    assert!(world.error.is_none(), "an abstention is not a failure");
}

fn assert_unavailable(world: &World, reason: UnavailabilityReason) {
    match world.outcome.as_ref().unwrap_or_else(|| panic!("no outcome (error: {:?})", world.error)) {
        QuestionOutcome::Unavailable { reason: actual } => {
            assert_eq!(*actual, reason, "unavailability reason");
        }
        other => panic!("expected an unavailability report, got {other:?}"),
    }
}

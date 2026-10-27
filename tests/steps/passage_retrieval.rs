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

#[given(regex = r"^an indexed library$")]
async fn indexed_library(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
}

#[given(regex = r"^the claim appears in indexed works of the genres fachbuch and sachbuch$")]
async fn claim_in_two_genres(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
}

#[given(regex = r"^an indexed library that holds no passage for the claim$")]
async fn library_without_claim(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
}

#[given(regex = r"^an indexed library where four passages reach the threshold$")]
async fn library_with_four(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
}

#[given(regex = r"^an indexed library where more than twenty passages reach the threshold$")]
async fn library_with_many(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
}

#[given(regex = r"^an index that is older than the freshness bound$")]
async fn stale_index(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
    world.make_stale().expect("backdate the index");
}

#[given(regex = r"^no edition is indexed$")]
async fn empty_index(world: &mut World) {
    world.ensure_empty_index().await.expect("empty index");
}

#[given(regex = r"^the library metadata listing, the accepted glossary and the index generation are recorded$")]
async fn record_library(world: &mut World) {
    world.ensure_indexed().await.expect("index the fixture library");
    world.record_library();
}

#[given(regex = r"^the last successful index time reported by the readiness probe is recorded$")]
async fn last_success_recorded(world: &mut World) {
    assert!(world.recorded.is_some(), "recording includes the last success time");
}

#[when(regex = r"^the agent submits that claim with scope genre=fachbuch$")]
async fn submit_with_genre(world: &mut World) {
    world
        .search(
            CLAIM,
            "de",
            Some(Scope {
                genre: Some("fachbuch".into()),
                ..Scope::default()
            }),
            None,
        )
        .await
        .expect("search");
}

#[when(regex = r#"^the agent submits a claim with a scope naming the work "([^"]+)"$"#)]
async fn submit_with_work(world: &mut World, work: String) {
    world
        .search(
            CLAIM,
            "de",
            Some(Scope {
                work_ids: vec![work],
                ..Scope::default()
            }),
            None,
        )
        .await
        .expect("search");
}

#[when(regex = r"^the agent submits the claim$")]
async fn submit_absent_claim(world: &mut World) {
    world.search(ABSENT, "de", None, None).await.expect("search");
}

#[when(regex = r"^the agent submits a claim$")]
async fn submit_a_claim(world: &mut World) {
    world.search(CLAIM, "de", None, None).await.expect("search");
}

#[when(regex = r"^the agent requests 20 passages$")]
async fn request_twenty(world: &mut World) {
    world.search(RARE, "de", None, Some(20)).await.expect("search");
}

#[when(regex = r"^the agent requests 50 passages$")]
async fn request_fifty(world: &mut World) {
    world.search(CLAIM, "de", None, Some(50)).await.expect("search");
}

#[when(regex = r"^the agent retrieves passages and widens context around one of them$")]
async fn retrieve_and_widen(world: &mut World) {
    world.search(CLAIM, "de", None, Some(5)).await.expect("search");
    let (_, returned, _, citations) = candidate(world);
    assert!(returned > 0, "a passage to widen");
    let citation = citations[0].clone();
    world
        .read(&citation.edition_id.0, citation.passage_index, Some(120))
        .await
        .expect("read");
}

#[when(regex = r"^the agent submits the same claim, scope and limit twice$")]
async fn submit_twice(world: &mut World) {
    let scope = Some(Scope {
        genre: Some("fachbuch".into()),
        ..Scope::default()
    });
    world.search(CLAIM, "de", scope.clone(), Some(5)).await.expect("first search");
    world.second_outcome = world.outcome.clone();
    world.search(CLAIM, "de", scope, Some(5)).await.expect("second search");
}

#[then(regex = r"^at least one returned passage comes from a work of the genre fachbuch$")]
async fn at_least_one_fachbuch(world: &mut World) {
    let (_, _, _, citations) = candidate(world);
    let works = works(world);
    assert!(
        citations.iter().any(|citation| works
            .get(&citation.work_id)
            .is_some_and(|work| work.genre == Some(Genre::Fachbuch))),
        "at least one fachbuch passage"
    );
}

#[then(regex = r"^every returned citation belongs to an indexed work of the genre fachbuch$")]
async fn every_fachbuch(world: &mut World) {
    let (_, _, _, citations) = candidate(world);
    let works = works(world);
    for citation in citations {
        let work = works.get(&citation.work_id).expect("indexed work");
        assert_eq!(work.genre, Some(Genre::Fachbuch), "citation genre for {}", work.id);
    }
}

#[then(regex = r"^no returned citation belongs to a work outside the genre fachbuch$")]
async fn none_outside(world: &mut World) {
    every_fachbuch(world).await;
}

#[then(regex = r"^the agent receives no evidence with reason unknown work$")]
async fn unknown_work(world: &mut World) {
    assert_abstention(world, "unknown_work");
}

#[then(regex = r"^no passage is returned$")]
async fn no_passage(world: &mut World) {
    match world.outcome.as_ref().expect("outcome") {
        QuestionOutcome::NoEvidence { .. } => {}
        other => panic!("expected no passage, got {other:?}"),
    }
}

#[then(regex = r"^the agent receives no evidence with reason no evidence$")]
async fn no_evidence(world: &mut World) {
    assert_abstention(world, "no_evidence");
}

#[then(regex = r"^the result is neither a failure nor an empty success$")]
async fn not_failure_not_empty(world: &mut World) {
    assert!(world.error.is_none());
    assert!(matches!(
        world.outcome.as_ref().expect("outcome"),
        QuestionOutcome::NoEvidence { .. }
    ));
}

#[then(regex = r"^the agent receives an unavailability report with reason stale index$")]
async fn stale_report(world: &mut World) {
    assert_unavailable(world, UnavailabilityReason::StaleIndex);
}

#[then(regex = r"^the agent receives an unavailability report with reason index unavailable$")]
async fn unavailable_report(world: &mut World) {
    assert_unavailable(world, UnavailabilityReason::IndexUnavailable);
}

#[then(regex = r"^the report is not a no-evidence result$")]
async fn report_not_abstention(world: &mut World) {
    assert!(matches!(
        world.outcome.as_ref().expect("outcome"),
        QuestionOutcome::Unavailable { .. }
    ));
}

#[then(regex = r"^the result holds 4 passages$")]
async fn holds_four(world: &mut World) {
    let (requested, returned, _, citations) = candidate(world);
    assert_eq!(requested, 20);
    assert_eq!(returned, 4);
    assert_eq!(citations.len(), 4);
}

#[then(regex = r"^the result is marked short$")]
async fn marked_short(world: &mut World) {
    let (_, _, short, _) = candidate(world);
    assert!(short, "short result");
}

#[then(regex = r"^no passage is padded to reach the requested number$")]
async fn not_padded(world: &mut World) {
    let (_, returned, _, citations) = candidate(world);
    assert_eq!(returned, citations.len());
}

#[then(regex = r"^the result holds 20 passages$")]
async fn holds_twenty(world: &mut World) {
    let (requested, returned, _, _) = candidate(world);
    assert_eq!(requested, 20);
    assert_eq!(returned, 20);
}

#[then(regex = r"^the result is not marked short$")]
async fn not_short(world: &mut World) {
    let (_, _, short, _) = candidate(world);
    assert!(!short, "not short");
}

#[then(regex = r"^the library metadata listing is unchanged$")]
async fn listing_unchanged(world: &mut World) {
    let recorded = world.recorded.as_ref().expect("recorded listing").0.clone();
    assert_eq!(world.listing(), recorded, "library listing");
}

#[then(regex = r"^the accepted glossary is unchanged$")]
async fn glossary_unchanged(world: &mut World) {
    assert_eq!(
        world.store().glossary_count().expect("glossary"),
        world.recorded_glossary
    );
}

#[then(regex = r"^the index generation and the last successful index time are unchanged$")]
async fn generation_unchanged(world: &mut World) {
    let (_, generation, last_success) = world.recorded.clone().expect("recorded");
    let status = world.store().status().expect("status");
    assert_eq!(status.library_generation, generation);
    assert_eq!(status.last_success_at, last_success);
}

#[then(regex = r"^both candidate-passage results carry the same citations in the same order$")]
async fn same_sequence(world: &mut World) {
    let keys = |outcome: &QuestionOutcome| match outcome {
        QuestionOutcome::CandidatePassages { passages, .. } => passages
            .iter()
            .map(|hit| {
                (
                    hit.citation.work_id.0.clone(),
                    hit.citation.edition_id.0.clone(),
                    hit.citation.passage_index,
                    hit.citation.char_start,
                    hit.citation.char_end,
                )
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let first = keys(world.outcome.as_ref().expect("second outcome"));
    let second = keys(world.second_outcome.as_ref().expect("first outcome"));
    assert_eq!(first, second);
}


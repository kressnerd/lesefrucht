//! `@e2e` step definitions: the real MCP surface over Streamable HTTP.

use cucumber::{given, then, when};
use serde_json::{Value, json};

use super::support::world::World;

fn passages(value: &Value) -> &Vec<Value> {
    value
        .get("passages")
        .and_then(|value| value.as_array())
        .expect("candidate passages")
}

fn citation(passage: &Value) -> &Value {
    passage.get("citation").expect("citation")
}

fn first_search(world: &World) -> &Value {
    world.e2e_search.as_ref().expect("search result")
}

fn first_context(world: &World) -> &Value {
    world.e2e_context.as_ref().expect("passage context")
}
#[given(regex = r"^the library holds at least one indexed edition$")]
async fn indexed_edition(world: &mut World) {
    world.ensure_mcp().await.expect("e2e stack ready");
}

#[given(regex = r"^the agent has retrieved a citation to an indexed edition$")]
async fn retrieved_citation(world: &mut World) {
    world.ensure_mcp().await.expect("e2e stack ready");
    if world.e2e_search.is_none() {
        let client = world.mcp.as_ref().expect("mcp client");
        let value = client
            .call(
                "search_passages",
                json!({
                    "claim": "Vertrauen in Organisationen",
                    "language": "de",
                    "scope": { "genre": "fachbuch" },
                    "limit": 5
                }),
            )
            .await
            .expect("search_passages");
        world.e2e_search = Some(value);
    }
    assert!(
        !passages(first_search(world)).is_empty(),
        "a retrieved citation"
    );
}

#[given(regex = r#"^the configured expansion translates the German term "([^"]+)" to the English term "([^"]+)"$"#)]
async fn expansion_configured(_world: &mut World, _source: String, _target: String) {
    // The e2e stack sets expansion.synonyms_file; asserted through the result.
}

#[when(regex = r#"^the agent submits the claim "([^"]+)" in German with scope genre=fachbuch, requesting 5 passages$"#)]
async fn submit_claim(world: &mut World, claim: String) {
    let client = world.mcp.as_ref().expect("mcp client");
    let value = client
        .call(
            "search_passages",
            json!({
                "claim": claim,
                "language": "de",
                "scope": { "genre": "fachbuch" },
                "limit": 5
            }),
        )
        .await
        .expect("search_passages");
    world.e2e_search = Some(value);
}

#[when(regex = r"^the agent requests wider context around the cited passage$")]
async fn widen(world: &mut World) {
    let search = first_search(world).clone();
    let passages = passages(&search);
    let first = passages.first().expect("a citation to widen");
    let citation = citation(first);
    let edition_id = citation
        .get("edition_id")
        .and_then(|value| value.as_str())
        .expect("edition id");
    let passage_index = citation
        .get("passage_index")
        .and_then(|value| value.as_u64())
        .expect("passage index");
    let client = world.mcp.as_ref().expect("mcp client");
    let value = client
        .call(
            "read_passage",
            json!({
                "edition_id": edition_id,
                "passage_index": passage_index,
                "window_chars": 400
            }),
        )
        .await
        .expect("read_passage");
    world.e2e_context = Some(value);
}

#[then(regex = r"^the agent receives a candidate-passage result$")]
async fn candidate_result(world: &mut World) {
    assert_eq!(
        first_search(world).get("kind").and_then(|value| value.as_str()),
        Some("candidate_passages")
    );
}

#[then(regex = r"^the result holds at least one passage and at most 5$")]
async fn holds_between_one_and_five(world: &mut World) {
    let count = passages(first_search(world)).len();
    assert!((1..=5).contains(&count), "returned {count} passages");
}

#[then(regex = r"^the result is marked short exactly when the returned number is smaller than the requested number$")]
async fn short_consistent(world: &mut World) {
    let value = first_search(world);
    let requested = value
        .get("requested")
        .and_then(|value| value.as_u64())
        .expect("requested");
    let returned = value
        .get("returned")
        .and_then(|value| value.as_u64())
        .expect("returned");
    let short = value
        .get("short")
        .and_then(|value| value.as_bool())
        .expect("short");
    assert_eq!(returned, passages(value).len() as u64);
    assert_eq!(short, returned < requested);
}

#[then(regex = r"^every returned passage carries a citation to an indexed edition, a score and the expansion used$")]
async fn carries_citation_score_expansion(world: &mut World) {
    for passage in passages(first_search(world)) {
        let citation = citation(passage);
        assert!(
            citation
                .get("edition_id")
                .and_then(|value| value.as_str())
                .is_some_and(|value| value.starts_with("calibre:"))
        );
        assert!(passage.get("score").and_then(|value| value.as_f64()).is_some());
        assert!(passage.get("expansion").and_then(|value| value.as_array()).is_some());
        assert!(passage.get("text").and_then(|value| value.as_str()).is_some());
    }
}

#[then(regex = r"^at least one returned citation belongs to an English edition$")]
async fn english_citation(world: &mut World) {
    assert!(
        passages(first_search(world))
            .iter()
            .any(|passage| citation(passage)
                .get("language")
                .and_then(|value| value.as_str())
                == Some("en")),
        "an English citation"
    );
}

#[then(regex = r#"^the expansion names the term "([^"]+)"$"#)]
async fn expansion_names(world: &mut World, term: String) {
    let terms = first_search(world)
        .get("expansion")
        .and_then(|value| value.get("expanded_terms"))
        .and_then(|value| value.as_array())
        .expect("expanded terms");
    assert!(terms.iter().any(|value| value.as_str() == Some(term.as_str())));
}

#[then(regex = r"^the passages are ordered by score, then work id in numeric order, then passage index$")]
async fn ordered(world: &mut World) {
    let mut previous: Option<(f64, u64, u64)> = None;
    for passage in passages(first_search(world)) {
        let score = passage
            .get("score")
            .and_then(|value| value.as_f64())
            .expect("score");
        let citation = citation(passage);
        let work_id = citation
            .get("work_id")
            .and_then(|value| value.as_str())
            .and_then(|value| value.parse::<u64>().ok())
            .expect("numeric work id");
        let passage_index = citation
            .get("passage_index")
            .and_then(|value| value.as_u64())
            .expect("passage index");
        if let Some((previous_score, previous_work, previous_index)) = previous {
            assert!(
                score < previous_score
                    || (score == previous_score
                        && (work_id > previous_work
                            || (work_id == previous_work && passage_index > previous_index))),
                "order violated"
            );
        }
        previous = Some((score, work_id, passage_index));
    }
}

#[then(regex = r"^the agent receives the surrounding text with the cited passage inside it$")]
async fn surrounding_text(world: &mut World) {
    assert_eq!(
        first_context(world).get("kind").and_then(|value| value.as_str()),
        Some("passage_context")
    );
    assert!(!first_context(world)
        .get("text")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .is_empty());
}

#[then(regex = r"^the returned text between the cited start and end offsets equals the text of the cited passage$")]
async fn cited_text_verbatim(world: &mut World) {
    let search = first_search(world).clone();
    let first = passages(&search).first().expect("first passage").clone();
    let expected = first
        .get("text")
        .and_then(|value| value.as_str())
        .expect("passage text");
    let context = first_context(world);
    let text = context
        .get("text")
        .and_then(|value| value.as_str())
        .expect("context text");
    let bounds_start = context
        .get("bounds")
        .and_then(|value| value.get("start_char"))
        .and_then(|value| value.as_u64())
        .expect("bounds start") as usize;
    let cited_start = context
        .get("cited")
        .and_then(|value| value.get("start_char"))
        .and_then(|value| value.as_u64())
        .expect("cited start") as usize;
    let cited_end = context
        .get("cited")
        .and_then(|value| value.get("end_char"))
        .and_then(|value| value.as_u64())
        .expect("cited end") as usize;
    let slice: String = text
        .chars()
        .skip(cited_start - bounds_start)
        .take(cited_end - cited_start)
        .collect();
    assert_eq!(&slice, expected);
}

#[then(regex = r"^the cited start offset is smaller than the cited end offset$")]
async fn cited_order(world: &mut World) {
    let context = first_context(world);
    let start = context
        .get("cited")
        .and_then(|value| value.get("start_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    let end = context
        .get("cited")
        .and_then(|value| value.get("end_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    assert!(start < end);
}

#[then(regex = r"^the bounds start at or before the cited start and end at or after the cited end$")]
async fn bounds_cover(world: &mut World) {
    let context = first_context(world);
    let bounds_start = context
        .get("bounds")
        .and_then(|value| value.get("start_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    let bounds_end = context
        .get("bounds")
        .and_then(|value| value.get("end_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    let cited_start = context
        .get("cited")
        .and_then(|value| value.get("start_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    let cited_end = context
        .get("cited")
        .and_then(|value| value.get("end_char"))
        .and_then(|value| value.as_u64())
        .unwrap();
    assert!(bounds_start <= cited_start);
    assert!(bounds_end >= cited_end);
}

#[then(regex = r"^the bounds name the same chapter as the cited passage$")]
async fn chapter_equal(world: &mut World) {
    let context = first_context(world);
    let bounds_chapter = context
        .get("bounds")
        .and_then(|value| value.get("chapter"))
        .expect("bounds chapter");
    let citation_chapter = context
        .get("citation")
        .and_then(|value| value.get("chapter"))
        .expect("citation chapter");
    assert_eq!(bounds_chapter, citation_chapter);
}


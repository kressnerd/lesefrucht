//! Deterministic OpenAI-compatible embedding twin.

use axum::{Json, Router, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct EmbeddingRequest {
    input: EmbeddingInput,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EmbeddingInput {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Serialize)]
struct EmbeddingResponse {
    object: &'static str,
    data: Vec<EmbeddingDatum>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct EmbeddingDatum {
    object: &'static str,
    index: usize,
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

#[derive(Clone, Copy)]
pub struct TwinState {
    pub dimension: usize,
}

/// Deterministic embedding: hashes whitespace-separated tokens into fixed
/// buckets, then L2-normalises. The same text always yields the same vector.
pub fn embed(text: &str, dimension: usize) -> Vec<f32> {
    let mut vector = vec![0.0f32; dimension];
    for token in text.split_whitespace() {
        let digest = blake_like(token.as_bytes());
        let bucket = (digest as usize) % dimension;
        vector[bucket] += 1.0;
    }
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

fn blake_like(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn router(state: TwinState) -> Router {
    Router::new()
        .route("/v1/models", get(models))
        .route("/v1/embeddings", post(embeddings))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(state)
}

async fn models() -> Json<Value> {
    Json(json!({
        "object": "list",
        "data": [{ "id": "deterministic", "object": "model", "owned_by": "lesefrucht-tests" }]
    }))
}

async fn embeddings(
    axum::extract::State(state): axum::extract::State<TwinState>,
    Json(request): Json<EmbeddingRequest>,
) -> Json<EmbeddingResponse> {
    let inputs = match request.input {
        EmbeddingInput::One(text) => vec![text],
        EmbeddingInput::Many(texts) => texts,
    };
    let prompt_tokens = inputs
        .iter()
        .map(|text| text.split_whitespace().count())
        .sum();
    let data = inputs
        .iter()
        .enumerate()
        .map(|(index, text)| EmbeddingDatum {
            object: "embedding",
            index,
            embedding: embed(text, state.dimension),
        })
        .collect();
    Json(EmbeddingResponse {
        object: "list",
        data,
        model: request
            .model
            .unwrap_or_else(|| "deterministic".to_string()),
        usage: Usage {
            prompt_tokens,
            total_tokens: prompt_tokens,
        },
    })
}

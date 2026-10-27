//! OpenAI-compatible embeddings, batched and dimension-checked.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("embedding request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("embedding response has {actual} dimensions, expected {expected}")]
    Dimension { actual: usize, expected: usize },
    #[error("embedding response is malformed: {0}")]
    Malformed(String),
}

#[derive(Debug, Clone)]
pub struct EmbeddingClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
    dimension: usize,
    batch_size: usize,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingDatum {
    embedding: Vec<f32>,
}

impl EmbeddingClient {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        dimension: usize,
        batch_size: usize,
        timeout_ms: u64,
    ) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_millis(timeout_ms))
                .build()
                .expect("embedding client"),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            dimension,
            batch_size: batch_size.max(1),
        }
    }

    pub async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        let mut vectors = Vec::with_capacity(texts.len());
        for batch in texts.chunks(self.batch_size) {
            let request = EmbeddingRequest {
                model: &self.model,
                input: batch,
            };
            let response: EmbeddingResponse = self
                .http
                .post(format!("{}/embeddings", self.base_url))
                .json(&request)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if response.data.len() != batch.len() {
                return Err(EmbedError::Malformed(format!(
                    "expected {} embeddings, received {}",
                    batch.len(),
                    response.data.len()
                )));
            }
            for datum in response.data {
                if datum.embedding.len() != self.dimension {
                    return Err(EmbedError::Dimension {
                        actual: datum.embedding.len(),
                        expected: self.dimension,
                    });
                }
                vectors.push(datum.embedding);
            }
        }
        Ok(vectors)
    }
}

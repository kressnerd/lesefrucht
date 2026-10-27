//! usearch HNSW over passage embeddings, with a JSON key map.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind, new_index};

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("usearch: {0}")]
    Usearch(String),
    #[error("vector map: {0}")]
    Io(#[from] std::io::Error),
    #[error("vector map json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    pub key: u64,
    pub uid: String,
    pub work_id: String,
    pub edition_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VectorMap {
    dimension: usize,
    records: Vec<VectorRecord>,
}

#[derive(Debug, Clone)]
pub struct VectorHit {
    pub uid: String,
    pub work_id: String,
    pub edition_id: String,
    pub similarity: f32,
}

pub struct VectorIndex {
    inner: Mutex<VectorInner>,
    map_path: PathBuf,
    index_path: PathBuf,
    dimension: usize,
}

struct VectorInner {
    index: Index,
    map: VectorMap,
    next_key: u64,
}

impl VectorIndex {
    pub fn open(path: &Path, dimension: usize) -> Result<Self, VectorError> {
        std::fs::create_dir_all(path)?;
        let index_path = path.join("vectors.usearch");
        let map_path = path.join("vectors.map.json");
        let map = if map_path.exists() {
            serde_json::from_slice(&std::fs::read(&map_path)?).unwrap_or_default()
        } else {
            VectorMap::default()
        };
        let index = if index_path.exists() {
            Index::restore(index_path.to_str().unwrap_or_default()).map_err(|error| VectorError::Usearch(error.to_string()))?
        } else {
            let options = IndexOptions {
                dimensions: dimension,
                metric: MetricKind::Cos,
                quantization: ScalarKind::F32,
                connectivity: 0,
                expansion_add: 0,
                expansion_search: 0,
                multi: false,
            };
            new_index(&options).map_err(|error| VectorError::Usearch(error.to_string()))?
        };
        index
            .reserve(map.records.len().max(16))
            .map_err(|error| VectorError::Usearch(error.to_string()))?;
        let next_key = map.records.iter().map(|record| record.key + 1).max().unwrap_or(1);
        Ok(Self {
            inner: Mutex::new(VectorInner {
                index,
                map: VectorMap {
                    dimension,
                    records: map.records,
                },
                next_key,
            }),
            map_path,
            index_path,
            dimension,
        })
    }

    pub fn add(
        &self,
        uid: &str,
        work_id: &str,
        edition_id: &str,
        vector: &[f32],
    ) -> Result<(), VectorError> {
        if vector.len() != self.dimension {
            return Err(VectorError::Usearch(format!(
                "vector dimension {} does not match {}",
                vector.len(),
                self.dimension
            )));
        }
        let mut inner = self.inner.lock().expect("vector index");
        let key = inner.next_key;
        inner.next_key += 1;
        if inner.index.size() + 1 > inner.index.capacity() {
            inner
                .index
                .reserve((inner.index.capacity() * 2).max(key as usize + 1))
                .map_err(|error| VectorError::Usearch(error.to_string()))?;
        }
        inner
            .index
            .add(key, vector)
            .map_err(|error| VectorError::Usearch(error.to_string()))?;
        inner.map.records.push(VectorRecord {
            key,
            uid: uid.to_string(),
            work_id: work_id.to_string(),
            edition_id: edition_id.to_string(),
        });
        Ok(())
    }

    pub fn remove_edition(&self, edition_id: &str) -> Result<(), VectorError> {
        let mut inner = self.inner.lock().expect("vector index");
        let (removed, kept): (Vec<_>, Vec<_>) = inner
            .map
            .records
            .drain(..)
            .partition(|record| record.edition_id == edition_id);
        for record in &removed {
            inner
                .index
                .remove(record.key)
                .map_err(|error| VectorError::Usearch(error.to_string()))?;
        }
        inner.map.records = kept;
        Ok(())
    }

    pub fn save(&self) -> Result<(), VectorError> {
        let inner = self.inner.lock().expect("vector index");
        inner
            .index
            .save(self.index_path.to_string_lossy().as_ref())
            .map_err(|error| VectorError::Usearch(error.to_string()))?;
        std::fs::write(&self.map_path, serde_json::to_vec_pretty(&inner.map)?)?;
        Ok(())
    }

    pub fn search(
        &self,
        query: &[f32],
        allowed: Option<&HashSet<String>>,
        limit: usize,
    ) -> Result<Vec<VectorHit>, VectorError> {
        if query.len() != self.dimension || limit == 0 {
            return Ok(Vec::new());
        }
        let inner = self.inner.lock().expect("vector index");
        let overfetch = limit.saturating_mul(4).max(limit);
        let matches = inner
            .index
            .search(query, overfetch)
            .map_err(|error| VectorError::Usearch(error.to_string()))?;
        let by_key: HashMap<u64, &VectorRecord> = inner
            .map
            .records
            .iter()
            .map(|record| (record.key, record))
            .collect();
        let mut hits = Vec::new();
        for (key, distance) in matches.keys.iter().zip(matches.distances.iter()) {
            let Some(record) = by_key.get(key) else {
                continue;
            };
            if let Some(work_ids) = allowed {
                if !work_ids.contains(&record.work_id) {
                    continue;
                }
            }
            hits.push(VectorHit {
                uid: record.uid.clone(),
                work_id: record.work_id.clone(),
                edition_id: record.edition_id.clone(),
                similarity: 1.0 - distance,
            });
            if hits.len() >= limit {
                break;
            }
        }
        Ok(hits)
    }

    pub fn stats(&self) -> (usize, usize) {
        let inner = self.inner.lock().expect("vector index");
        (inner.map.records.len(), inner.index.size())
    }
}

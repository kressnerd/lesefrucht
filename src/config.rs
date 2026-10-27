//! Configuration: TOML plus `LESEFRUCHT__<SECTION>__<KEY>` overrides.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read configuration {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot parse configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub mcp: McpConfig,
    pub library: LibraryConfig,
    pub edition: EditionConfig,
    pub extraction: ExtractionConfig,
    pub index: IndexConfig,
    pub chunk: ChunkConfig,
    pub embedding: EmbeddingConfig,
    pub expansion: ExpansionConfig,
    pub retrieval: RetrievalConfig,
    pub read: ReadConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct McpConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LibraryConfig {
    pub calibredb_binary: String,
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EditionConfig {
    pub format_preference: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ExtractionConfig {
    pub supported: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct IndexConfig {
    pub dir: PathBuf,
    pub freshness_bound_hours: u64,
    pub generation_cache_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ChunkConfig {
    pub max_chars: usize,
    pub overlap_chars: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    pub base_url: String,
    pub model: String,
    pub dimension: usize,
    pub batch_size: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ExpansionConfig {
    pub synonyms_file: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RetrievalConfig {
    pub limit_default: usize,
    pub limit_max: usize,
    pub lexical_min_score: f32,
    pub vector_min_cosine: f32,
    pub rrf_k: f32,
    pub vector_overfetch: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ReadConfig {
    pub max_window_chars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            mcp: McpConfig::default(),
            library: LibraryConfig::default(),
            edition: EditionConfig::default(),
            extraction: ExtractionConfig::default(),
            index: IndexConfig::default(),
            chunk: ChunkConfig::default(),
            embedding: EmbeddingConfig::default(),
            expansion: ExpansionConfig::default(),
            retrieval: RetrievalConfig::default(),
            read: ReadConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8787".into(),
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self { path: "/mcp".into() }
    }
}

impl Default for LibraryConfig {
    fn default() -> Self {
        Self {
            calibredb_binary: "calibredb".into(),
            url: String::new(),
            timeout_ms: 30_000,
        }
    }
}

impl Default for EditionConfig {
    fn default() -> Self {
        Self {
            format_preference: vec![
                "epub".into(),
                "azw3".into(),
                "mobi".into(),
                "pdf".into(),
            ],
        }
    }
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            supported: vec!["epub".into(), "pdf".into()],
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("data/index"),
            freshness_bound_hours: 168,
            generation_cache_ms: 5_000,
        }
    }
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_chars: 1_800,
            overlap_chars: 200,
        }
    }
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            model: String::new(),
            dimension: 0,
            batch_size: 32,
            timeout_ms: 10_000,
        }
    }
}

impl Default for ExpansionConfig {
    fn default() -> Self {
        Self {
            synonyms_file: PathBuf::new(),
        }
    }
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            limit_default: 5,
            limit_max: 20,
            lexical_min_score: 0.15,
            vector_min_cosine: 0.20,
            rrf_k: 60.0,
            vector_overfetch: 4,
        }
    }
}

impl Default for ReadConfig {
    fn default() -> Self {
        Self {
            max_window_chars: 4_000,
        }
    }
}

impl Config {
    /// Load from an optional TOML file, then apply environment overrides.
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let mut config = match path {
            Some(path) => {
                let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
                    path: path.to_path_buf(),
                    source,
                })?;
                toml::from_str(&raw)?
            }
            None => Self::default(),
        };
        apply_env(&mut config)?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.library.url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "library.url is required and has no default".into(),
            ));
        }
        if self.embedding.base_url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "embedding.base_url is required and has no default".into(),
            ));
        }
        if self.embedding.model.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "embedding.model is required and has no default".into(),
            ));
        }
        if self.embedding.dimension == 0 {
            return Err(ConfigError::Invalid(
                "embedding.dimension is required and must be positive".into(),
            ));
        }
        if self.retrieval.limit_max == 0 || self.retrieval.limit_default == 0 {
            return Err(ConfigError::Invalid(
                "retrieval.limit_default and retrieval.limit_max must be positive".into(),
            ));
        }
        self.server_socket()?;
        let probe = Command::new(&self.library.calibredb_binary)
            .arg("--version")
            .output();
        match probe {
            Ok(_) => Ok(()),
            Err(source) => Err(ConfigError::Invalid(format!(
                "library.calibredb_binary `{}` cannot be executed: {source}",
                self.library.calibredb_binary
            ))),
        }
    }

    pub fn server_socket(&self) -> Result<SocketAddr, ConfigError> {
        self.server
            .bind
            .parse()
            .map_err(|_| ConfigError::Invalid(format!("server.bind `{}` is not a socket address", self.server.bind)))
    }
}

fn apply_env(config: &mut Config) -> Result<(), ConfigError> {
    for (key, raw) in std::env::vars() {
        let Some(rest) = key.strip_prefix("LESEFRUCHT__") else {
            continue;
        };
        let parts: Vec<&str> = rest.split("__").collect();
        if parts.len() != 2 {
            continue;
        }
        let section = parts[0].to_ascii_lowercase();
        let field = parts[1].to_ascii_lowercase();

        match (section.as_str(), field.as_str()) {
            ("server", "bind") => config.server.bind = raw,
            ("mcp", "path") => config.mcp.path = raw,
            ("library", "calibredb_binary") => config.library.calibredb_binary = raw,
            ("library", "url") => config.library.url = raw,
            ("library", "timeout_ms") => config.library.timeout_ms = parse_env(&raw, &key)?,
            ("edition", "format_preference") => {
                config.edition.format_preference = split_list(&raw)
            }
            ("extraction", "supported") => config.extraction.supported = split_list(&raw),
            ("index", "dir") => config.index.dir = PathBuf::from(raw),
            ("index", "freshness_bound_hours") => {
                config.index.freshness_bound_hours = parse_env(&raw, &key)?
            }
            ("index", "generation_cache_ms") => {
                config.index.generation_cache_ms = parse_env(&raw, &key)?
            }
            ("chunk", "max_chars") => config.chunk.max_chars = parse_env(&raw, &key)?,
            ("chunk", "overlap_chars") => config.chunk.overlap_chars = parse_env(&raw, &key)?,
            ("embedding", "base_url") => config.embedding.base_url = raw,
            ("embedding", "model") => config.embedding.model = raw,
            ("embedding", "dimension") => config.embedding.dimension = parse_env(&raw, &key)?,
            ("embedding", "batch_size") => config.embedding.batch_size = parse_env(&raw, &key)?,
            ("embedding", "timeout_ms") => config.embedding.timeout_ms = parse_env(&raw, &key)?,
            ("expansion", "synonyms_file") => {
                config.expansion.synonyms_file = PathBuf::from(raw)
            }
            ("retrieval", "limit_default") => {
                config.retrieval.limit_default = parse_env(&raw, &key)?
            }
            ("retrieval", "limit_max") => config.retrieval.limit_max = parse_env(&raw, &key)?,
            ("retrieval", "lexical_min_score") => {
                config.retrieval.lexical_min_score = parse_env(&raw, &key)?
            }
            ("retrieval", "vector_min_cosine") => {
                config.retrieval.vector_min_cosine = parse_env(&raw, &key)?
            }
            ("retrieval", "rrf_k") => config.retrieval.rrf_k = parse_env(&raw, &key)?,
            ("retrieval", "vector_overfetch") => {
                config.retrieval.vector_overfetch = parse_env(&raw, &key)?
            }
            ("read", "max_window_chars") => config.read.max_window_chars = parse_env(&raw, &key)?,
            _ => {}
        }
    }
    Ok(())
}

fn parse_env<T: std::str::FromStr>(raw: &str, key: &str) -> Result<T, ConfigError> {
    raw.parse().map_err(|_| {
        ConfigError::Invalid(format!("environment override {key} is not valid"))
    })
}

fn split_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_design() {
        let config = Config::default();
        assert_eq!(config.read.max_window_chars, 4_000);
        assert_eq!(config.retrieval.limit_max, 20);
        assert_eq!(config.chunk.max_chars, 1_800);
    }
}

//! Scenario world: a fresh index over the fixture library, the service, and
//! the observable results a scenario asserts on.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::OnceCell;

use lesefrucht::config::Config;
use lesefrucht::domain::{PassageContext, QuestionOutcome};
use lesefrucht::index::IndexStore;
use lesefrucht::indexing::Indexer;
use lesefrucht::retrieval::{ReadRequest, SearchRequest, SearchService};
use tempfile::TempDir;

#[derive(Default, cucumber::World)]
pub struct World {
    pub temp: Option<TempDir>,
    pub library: PathBuf,
    pub config: Option<Config>,
    pub store: Option<Arc<IndexStore>>,
    pub service: Option<Arc<SearchService>>,
    pub outcome: Option<QuestionOutcome>,
    pub second_outcome: Option<QuestionOutcome>,
    pub error: Option<String>,
    pub context: Option<PassageContext>,
    pub recorded: Option<(String, Option<String>, Option<u64>)>,
    pub recorded_glossary: usize,
    pub named_work: Option<String>,
    pub mcp: Option<super::mcp_client::McpClient>,
    pub e2e_search: Option<serde_json::Value>,
    pub e2e_context: Option<serde_json::Value>,
    pub twin: Option<tokio::task::JoinHandle<()>>,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("library", &self.library)
            .field("has_service", &self.service.is_some())
            .finish()
    }
}

impl World {
    pub fn fixture_library() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/library")
    }

    pub fn synonyms() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/synonyms.toml")
    }

    pub async fn ensure_indexed(&mut self) -> anyhow::Result<()> {
        if self.service.is_some() {
            return Ok(());
        }
        let shared = shared().await;
        self.library = shared.library.clone();
        self.config = Some(shared.config.clone());
        self.store = Some(Arc::clone(&shared.store));
        self.service = Some(Arc::new(SearchService::new(
            Arc::clone(&shared.store),
            shared.config.clone(),
        )));
        Ok(())
    }

    pub async fn ensure_empty_index(&mut self) -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let library = temp.path().join("library");
        copy_dir(&Self::fixture_library(), &library)?;
        let config = self.config_for(&temp, &library)?;
        let store = Arc::new(IndexStore::open(&config)?);
        let service = Arc::new(SearchService::new(Arc::clone(&store), config.clone()));
        self.temp = Some(temp);
        self.library = library;
        self.config = Some(config);
        self.store = Some(store);
        self.service = Some(service);
        Ok(())
    }

    pub fn config_for(&mut self, temp: &TempDir, library: &Path) -> anyhow::Result<Config> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        drop(listener);
        let dimension = 64;
        let twin = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(address).await.unwrap();
            axum::serve(
                listener,
                openai_compatible::router(openai_compatible::TwinState { dimension }),
            )
            .await
            .unwrap();
        });
        self.twin = Some(twin);
        let mut config = Config::default();
        config.library.url = library.to_string_lossy().to_string();
        config.library.calibredb_binary =
            std::env::var("CALIBREDB_BINARY").unwrap_or_else(|_| "calibredb".into());
        config.embedding.base_url = format!("http://{address}/v1");
        config.embedding.model = "deterministic".into();
        config.embedding.dimension = dimension;
        config.index.dir = temp.path().join("index");
        config.expansion.synonyms_file = Self::synonyms();
        config.read.max_window_chars = 120;
        config.retrieval.vector_min_cosine = 0.99;
        config.retrieval.lexical_min_score = 0.05;
        Ok(config)
    }

    pub fn store(&self) -> Arc<IndexStore> {
        Arc::clone(self.store.as_ref().expect("indexed world"))
    }

    pub fn service(&self) -> Arc<SearchService> {
        Arc::clone(self.service.as_ref().expect("service world"))
    }

    pub async fn search(
        &mut self,
        claim: &str,
        language: &str,
        scope: Option<lesefrucht::retrieval::scope::Scope>,
        limit: Option<usize>,
    ) -> anyhow::Result<()> {
        let service = self.service();
        match service
            .search(SearchRequest {
                claim: claim.to_string(),
                language: language.to_string(),
                scope,
                limit,
            })
            .await
        {
            Ok(outcome) => {
                self.outcome = Some(outcome);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error_reason(&error));
                self.outcome = None;
            }
        }
        Ok(())
    }

    pub async fn read(
        &mut self,
        edition_id: &str,
        passage_index: u32,
        window_chars: Option<usize>,
    ) -> anyhow::Result<()> {
        let service = self.service();
        match service
            .read_passage(ReadRequest {
                edition_id: edition_id.to_string(),
                passage_index,
                window_chars,
            })
            .await
        {
            Ok(context) => {
                self.context = Some(context);
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error_reason(&error));
                self.context = None;
            }
        }
        Ok(())
    }

    pub fn listing(&self) -> String {
        let binary = self
            .config
            .as_ref()
            .map(|config| config.library.calibredb_binary.clone())
            .unwrap_or_else(|| "calibredb".into());
        std::process::Command::new(binary)
            .args([
                "list",
                "--with-library",
                &self.library.to_string_lossy(),
                "--for-machine",
                "--fields",
                "all",
            ])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            .unwrap_or_default()
    }

    pub fn record_library(&mut self) {
        let store = self.store();
        let status = store.status().ok();
        self.recorded = Some((
            self.listing(),
            status.as_ref().and_then(|status| status.library_generation.clone()),
            status.and_then(|status| status.last_success_at),
        ));
        self.recorded_glossary = store.glossary_count().unwrap_or_default();
    }

    pub fn make_stale(&mut self) -> anyhow::Result<()> {
        let mut config = self.config.clone().expect("config");
        config.index.freshness_bound_hours = 0;
        self.service = Some(Arc::new(SearchService::new(self.store(), config.clone())));
        self.config = Some(config);
        Ok(())
    }

    pub fn break_library(&mut self) -> anyhow::Result<()> {
        let mut config = self.config.clone().expect("config");
        config.library.url = "http://127.0.0.1:9".into();
        let store = self.store();
        self.service = Some(Arc::new(SearchService::new(store, config.clone())));
        self.config = Some(config);
        Ok(())
    }

    pub async fn ensure_light(&mut self) -> anyhow::Result<()> {
        if self.service.is_some() {
            return Ok(());
        }
        let temp = tempfile::tempdir()?;
        let mut config = Config::default();
        config.library.url = "http://127.0.0.1:9".into();
        config.embedding.base_url = "http://127.0.0.1:9/v1".into();
        config.embedding.model = "deterministic".into();
        config.embedding.dimension = 64;
        config.index.dir = temp.path().join("index");
        let store = Arc::new(IndexStore::open(&config)?);
        let service = Arc::new(SearchService::new(Arc::clone(&store), config.clone()));
        self.temp = Some(temp);
        self.config = Some(config);
        self.store = Some(store);
        self.service = Some(service);
        Ok(())
    }

    pub async fn ensure_mcp(&mut self) -> anyhow::Result<()> {
        if self.mcp.is_some() {
            return Ok(());
        }
        let endpoint = std::env::var("LESEFRUCHT_E2E_MCP")
            .unwrap_or_else(|_| "http://127.0.0.1:8787/mcp".into());
        let health = endpoint.replace("/mcp", "/healthz");
        let http = reqwest::Client::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
        loop {
            if let Ok(response) = http.get(&health).send().await {
                if let Ok(value) = response.json::<serde_json::Value>().await {
                    let indexed = value
                        .get("indexed_editions")
                        .and_then(|value| value.as_u64())
                        .unwrap_or_default();
                    let ready = value
                        .get("index_state")
                        .and_then(|value| value.as_str())
                        == Some("ready");
                    if indexed >= 6 && ready {
                        break;
                    }
                }
            }
            if std::time::Instant::now() > deadline {
                anyhow::bail!("the e2e stack did not become ready");
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        let client = super::mcp_client::McpClient::connect(&endpoint).await?;
        self.mcp = Some(client);
        Ok(())
    }
}

fn error_reason(error: &lesefrucht::retrieval::ServiceError) -> String {
    use lesefrucht::retrieval::ServiceError;
    match error {
        ServiceError::InvalidArguments(_) => "invalid_arguments".into(),
        ServiceError::InvalidCitation => "invalid_citation".into(),
        other => other.to_string(),
    }
}

fn copy_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

struct Shared {
    _temp: TempDir,
    library: PathBuf,
    config: Config,
    store: Arc<IndexStore>,
    _twin: tokio::task::JoinHandle<()>,
}

static SHARED: OnceCell<Shared> = OnceCell::const_new();

async fn shared() -> &'static Shared {
    SHARED
        .get_or_init(|| async {
            let temp = tempfile::tempdir().expect("shared temp");
            let library = temp.path().join("library");
            copy_dir(&World::fixture_library(), &library).expect("copy fixture library");
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
            let address = listener.local_addr().expect("address");
            drop(listener);
            let dimension = 64;
            let twin = tokio::spawn(async move {
                let listener = tokio::net::TcpListener::bind(address).await.unwrap();
                axum::serve(
                    listener,
                    openai_compatible::router(openai_compatible::TwinState { dimension }),
                )
                .await
                .unwrap();
            });
            let mut config = Config::default();
            config.library.url = library.to_string_lossy().to_string();
            config.library.calibredb_binary =
                std::env::var("CALIBREDB_BINARY").unwrap_or_else(|_| "calibredb".into());
            config.embedding.base_url = format!("http://{address}/v1");
            config.embedding.model = "deterministic".into();
            config.embedding.dimension = dimension;
            config.index.dir = temp.path().join("index");
            config.expansion.synonyms_file = World::synonyms();
            config.read.max_window_chars = 120;
            config.retrieval.vector_min_cosine = 0.99;
            config.retrieval.lexical_min_score = 0.05;
            let store = Arc::new(IndexStore::open(&config).expect("shared store"));
            Indexer::new(Arc::clone(&store), config.clone())
                .run()
                .await
                .expect("shared index");
            Shared {
                _temp: temp,
                library,
                config,
                store,
                _twin: twin,
            }
        })
        .await
}

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use lesefrucht::config::Config;
use lesefrucht::index::IndexStore;
use lesefrucht::indexing::spawn_initial_indexing;
use lesefrucht::mcp;
use lesefrucht::retrieval::SearchService;

#[derive(Debug, Parser)]
#[command(name = "lesefrucht", version, about = "Cite passages from a calibre library")]
struct Args {
    /// Path to lesefrucht.toml.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::load(args.config.as_deref())?;
    lesefrucht::telemetry::init();
    let store = Arc::new(IndexStore::open(&config)?);
    spawn_initial_indexing(Arc::clone(&store), config.clone());
    let service = Arc::new(SearchService::new(Arc::clone(&store), config.clone()));
    let listener = tokio::net::TcpListener::bind(config.server_socket()?).await?;
    tracing::info!(address = %config.server.bind, "lesefrucht listening");
    let app = mcp::router(service, store, &config.mcp.path);
    axum::serve(listener, app).await?;
    Ok(())
}

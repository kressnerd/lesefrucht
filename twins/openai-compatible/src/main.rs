use std::net::SocketAddr;

use openai_compatible::{TwinState, router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8789);
    let dimension = std::env::var("EMBEDDING_DIMENSION")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(64);
    let app = router(TwinState { dimension });
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind embedding twin");
    tracing::info!(%address, %dimension, "embedding twin listening");
    axum::serve(listener, app)
        .await
        .expect("serve embedding twin");
}

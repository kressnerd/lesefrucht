//! In-process deterministic embedding endpoint for non-e2e scenarios.

use tokio::task::JoinHandle;

/// Bind an ephemeral loopback port and serve the twin; returns its base URL.
pub async fn spawn(dimension: usize) -> anyhow::Result<(JoinHandle<()>, String)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let twin = tokio::spawn(async move {
        axum::serve(
            listener,
            openai_compatible::router(openai_compatible::TwinState { dimension }),
        )
        .await
        .expect("serve embedding twin");
    });
    Ok((twin, format!("http://{address}")))
}

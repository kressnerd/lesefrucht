//! rmcp client used by the `@e2e` steps.

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use rmcp::RoleClient;
use serde_json::Value;

pub struct McpClient {
    inner: RunningService<RoleClient, ()>,
}

impl McpClient {
    pub async fn connect(endpoint: &str) -> anyhow::Result<Self> {
        let transport = StreamableHttpClientTransport::from_config(
            StreamableHttpClientTransportConfig::with_uri(endpoint.to_string()),
        );
        let inner = ().serve(transport).await?;
        Ok(Self { inner })
    }

    pub async fn call(&self, tool: &str, arguments: Value) -> anyhow::Result<Value> {
        let arguments = arguments
            .as_object()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("tool arguments must be an object"))?;
        let result = self
            .inner
            .call_tool(CallToolRequestParams::new(tool.to_string()).with_arguments(arguments))
            .await?;
        result
            .structured_content
            .ok_or_else(|| anyhow::anyhow!("the tool returned no structured content"))
    }
}

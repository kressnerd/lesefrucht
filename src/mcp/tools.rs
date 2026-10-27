//! The two tools: `search_passages` and `read_passage`.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerConfig};
use rmcp::{Json, ServerHandler, tool, tool_handler, tool_router};

use crate::domain::{PassageContext, QuestionOutcome};
use crate::retrieval::{ReadRequest, SearchRequest, SearchService};

use super::errors;

#[derive(Clone)]
pub struct LesefruchtServer {
    tool_router: ToolRouter<Self>,
    service: Arc<SearchService>,
}

#[tool_router]
impl LesefruchtServer {
    pub fn new(service: Arc<SearchService>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            service,
        }
    }

    #[tool(
        description = "Submit a claim or phrase to the indexed library and receive up to twenty candidate passages with citations, scores and the expansion used, or an abstention, or an unavailability report."
    )]
    async fn search_passages(
        &self,
        Parameters(request): Parameters<SearchRequest>,
    ) -> Result<Json<QuestionOutcome>, rmcp::model::ErrorData> {
        self.service
            .search(request)
            .await
            .map(Json)
            .map_err(errors::map)
    }

    #[tool(
        description = "Read the exact passage at a citation and optionally widen the surrounding context within its chapter."
    )]
    async fn read_passage(
        &self,
        Parameters(request): Parameters<ReadRequest>,
    ) -> Result<Json<PassageContext>, rmcp::model::ErrorData> {
        self.service
            .read_passage(request)
            .await
            .map(Json)
            .map_err(errors::map)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for LesefruchtServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("lesefrucht", env!("CARGO_PKG_VERSION"))
                    .with_title("lesefrucht")
                    .with_description("Cite passages from a private calibre library"),
            )
    }
}

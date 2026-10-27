//! Protocol errors versus tagged results.

use rmcp::model::ErrorData;
use serde_json::json;

use crate::retrieval::ServiceError;

pub fn map(error: ServiceError) -> ErrorData {
    match error {
        ServiceError::InvalidArguments(message) => {
            ErrorData::invalid_params(message, Some(json!({ "reason": "invalid_arguments" })))
        }
        ServiceError::InvalidCitation => ErrorData::invalid_params(
            "unknown edition or passage",
            Some(json!({ "reason": "invalid_citation" })),
        ),
        other => ErrorData::internal_error(other.to_string(), None),
    }
}

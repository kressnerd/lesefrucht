//! The agent surface: MCP over Streamable HTTP.

pub mod dto;
pub mod errors;
pub mod server;
pub mod tools;

pub use server::router;
pub use tools::LesefruchtServer;

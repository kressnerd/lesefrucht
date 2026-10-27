//! Retrieval: expansion, scope, both paths, fusion, outcomes, context.

pub mod expand;
pub mod fuse;
pub mod lexical;
pub mod scope;
pub mod service;
pub mod vector;

pub use service::{ReadRequest, SearchRequest, SearchService, ServiceError};

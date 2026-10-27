//! The calibre library interface and edition selection.

pub mod calibredb;
pub mod discovery;
pub mod selection;

pub use calibredb::{Calibredb, LibraryError, ListingBook};
pub use discovery::{discovery, LibrarySnapshot};
pub use selection::select_format;

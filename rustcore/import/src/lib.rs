//! Playlist import: file/link parsers behind a common `PlaylistSource`
//! trait, plus fuzzy catalog matching with an import report.

mod error;
pub mod importer;
mod matching;
pub mod parsers;
pub mod report;
mod source;

pub use error::ImportError;
pub use matching::TrackMatcher;
pub use importer::PlaylistImporter;
pub use report::ImportReport;
pub use source::{PlaylistSource, PlaylistSourceType, RawPlaylist};

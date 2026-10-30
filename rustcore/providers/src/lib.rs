//! Pluggable music sources. Each provider is user-configured; the app
//! ships no built-in access to copyrighted audio content.

pub mod local;
pub mod musicbrainz;
pub mod registry;

pub use local::LocalLibraryProvider;
pub use musicbrainz::MusicBrainzProvider;
pub use registry::ProviderRegistry;

//! Domain layer: core entities, provider abstraction, track matching.

pub mod entities;
pub mod matching;
pub mod provider;

pub use entities::*;
pub use matching::*;
pub use provider::*;

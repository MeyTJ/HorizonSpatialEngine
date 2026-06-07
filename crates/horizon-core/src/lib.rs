//! Spatial compute engine — no transport or serialization dependencies.
//!
//! All geometry is consumed via zero-copy archived views from `horizon-storage`.
//! This crate must never depend on gRPC, Protobuf, or any network stack.

mod engine;
mod error;
mod query;
mod spatial;

pub use engine::{QueryResult, SpatialEngine};
pub use error::CoreError;
pub use query::{AccessibilityResult, IntersectionResult, QueryBounds, SpatialQuery};
pub use spatial::{SharedSpatialIndex, SpatialEntry, SpatialIndex};

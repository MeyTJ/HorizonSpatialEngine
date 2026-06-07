//! Spatial compute engine — no transport or serialization dependencies.
//!
//! All geometry is consumed via zero-copy archived views from `horizon-storage`.
//! This crate must never depend on gRPC, Protobuf, or any network stack.

mod engine;
mod error;
mod query;
mod spatial;
mod visual_justice_metrics;

pub use engine::{QueryResult, SpatialEngine};
pub use error::CoreError;
pub use query::{AccessibilityResult, IntersectionResult, QueryBounds, SpatialQuery};
pub use spatial::{SharedSpatialIndex, SpatialEntry, SpatialIndex};
pub use visual_justice_metrics::{
    calculate_horizon_blockage, HorizonBlockageResult, COASTAL_HIGHRISE_MIN_HEIGHT,
    HORIZON_RAY_COUNT,
};

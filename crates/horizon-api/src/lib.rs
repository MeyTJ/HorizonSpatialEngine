//! Transport-agnostic API boundary.
//!
//! The gRPC layer depends exclusively on this crate. It must never import
//! `horizon-core`, `horizon-storage`, or `horizon-geometry` directly.

mod dto;
mod service;

pub use dto::{
    AccessibilityRequest, AccessibilityResponse, BoundingBox, IntersectRequest,
    IntersectResponse, LoadDatasetRequest, LoadDatasetResponse, ServiceError,
};
pub use service::SpatialService;

//! Generated Protobuf types and gRPC service stubs.

pub mod spatial {
    pub mod v1 {
        tonic::include_proto!("horizon.spatial.v1");
    }
}

pub use spatial::v1::spatial_service_client::SpatialServiceClient;
pub use spatial::v1::spatial_service_server::{SpatialService, SpatialServiceServer};
pub use spatial::v1::{
    AccessibilityRequest, AccessibilityResponse, BoundingBox, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse,
};

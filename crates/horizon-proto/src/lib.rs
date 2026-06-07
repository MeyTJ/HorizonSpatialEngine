//! Generated Protobuf types and gRPC service stubs.

pub mod spatial {
    pub mod v1 {
        tonic::include_proto!("horizon.spatial.v1");
    }
}

pub mod spatial_compute {
    pub mod v1 {
        tonic::include_proto!("horizon.spatial_compute.v1");
    }
}

pub use spatial::v1::spatial_service_client::SpatialServiceClient;
pub use spatial::v1::spatial_service_server::{SpatialService, SpatialServiceServer};
pub use spatial::v1::{
    AccessibilityRequest, AccessibilityResponse, BoundingBox, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse,
};

pub use spatial_compute::v1::spatial_compute_service_client::SpatialComputeServiceClient;
pub use spatial_compute::v1::spatial_compute_service_server::{
    SpatialComputeService, SpatialComputeServiceServer,
};
pub use spatial_compute::v1::{
    CalculateHorizonAccessRequest, CalculateHorizonAccessResponse, Coordinate, LineString,
};

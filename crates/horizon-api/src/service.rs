use async_trait::async_trait;

use crate::{
    AccessibilityRequest, AccessibilityResponse, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse, ServiceError,
};

/// Compute service contract consumed by the transport layer.
///
/// Implementations live outside `horizon-transport` to preserve decoupling.
#[async_trait]
pub trait SpatialService: Send + Sync + 'static {
    async fn load_dataset(
        &self,
        request: LoadDatasetRequest,
    ) -> Result<LoadDatasetResponse, ServiceError>;

    async fn intersect(
        &self,
        request: IntersectRequest,
    ) -> Result<IntersectResponse, ServiceError>;

    async fn accessibility(
        &self,
        request: AccessibilityRequest,
    ) -> Result<AccessibilityResponse, ServiceError>;
}

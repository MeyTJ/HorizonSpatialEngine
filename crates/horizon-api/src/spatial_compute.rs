use async_trait::async_trait;

use crate::{CalculateHorizonAccessRequest, CalculateHorizonAccessResponse, ServiceError};

/// Visual justice compute contract consumed by the gRPC transport layer.
#[async_trait]
pub trait SpatialComputeService: Send + Sync + 'static {
    async fn calculate_horizon_access(
        &self,
        request: CalculateHorizonAccessRequest,
    ) -> Result<CalculateHorizonAccessResponse, ServiceError>;
}

use std::sync::Arc;

use horizon_api::SpatialComputeService;
use horizon_proto::spatial_compute_service_server::SpatialComputeService as GrpcSpatialComputeService;
use horizon_proto::{
    CalculateHorizonAccessRequest, CalculateHorizonAccessResponse, SpatialComputeServiceServer,
};
use tonic::{Request, Response, Status};

use crate::spatial_compute_convert;
use crate::traceparent;

pub struct GrpcSpatialComputeServiceImpl {
    pub inner: Arc<dyn SpatialComputeService>,
}

#[tonic::async_trait]
impl GrpcSpatialComputeService for GrpcSpatialComputeServiceImpl {
    async fn calculate_horizon_access(
        &self,
        request: Request<CalculateHorizonAccessRequest>,
    ) -> Result<Response<CalculateHorizonAccessResponse>, Status> {
        let span = traceparent::span_from_metadata(request.metadata(), "CalculateHorizonAccess");
        let _guard = span.enter();

        let api_req = spatial_compute_convert::horizon_access_req(request.into_inner())?;
        let api_res = self
            .inner
            .calculate_horizon_access(api_req)
            .await
            .map_err(spatial_compute_convert::service_error)?;

        Ok(Response::new(spatial_compute_convert::horizon_access_res(
            api_res,
        )))
    }
}

pub fn spatial_compute_service(
    inner: Arc<dyn SpatialComputeService>,
) -> SpatialComputeServiceServer<GrpcSpatialComputeServiceImpl> {
    SpatialComputeServiceServer::new(GrpcSpatialComputeServiceImpl { inner })
}

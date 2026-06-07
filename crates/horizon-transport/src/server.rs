use std::net::SocketAddr;
use std::sync::Arc;

use horizon_api::SpatialService;
use horizon_proto::spatial_service_server::SpatialService as GrpcSpatialService;
use horizon_proto::{
    AccessibilityRequest, AccessibilityResponse, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse, SpatialServiceServer,
};
use tonic::{Request, Response, Status};
use tracing::info;

use crate::convert;

/// gRPC server configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub listen_addr: SocketAddr,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:50051".parse().expect("valid default listen address"),
        }
    }
}

/// Start the gRPC server and block until shutdown.
pub async fn serve(
    config: TransportConfig,
    service: Arc<dyn SpatialService>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let grpc = GrpcSpatialServiceImpl { inner: service };
    info!(addr = %config.listen_addr, "starting gRPC transport");

    tonic::transport::Server::builder()
        .add_service(SpatialServiceServer::new(grpc))
        .serve(config.listen_addr)
        .await?;

    Ok(())
}

struct GrpcSpatialServiceImpl {
    inner: Arc<dyn SpatialService>,
}

#[tonic::async_trait]
impl GrpcSpatialService for GrpcSpatialServiceImpl {
    async fn load_dataset(
        &self,
        request: Request<LoadDatasetRequest>,
    ) -> Result<Response<LoadDatasetResponse>, Status> {
        let api_req = convert::load_dataset_req(request.into_inner());
        let api_res = self
            .inner
            .load_dataset(api_req)
            .await
            .map_err(convert::service_error)?;
        Ok(Response::new(convert::load_dataset_res(api_res)))
    }

    async fn intersect(
        &self,
        request: Request<IntersectRequest>,
    ) -> Result<Response<IntersectResponse>, Status> {
        let api_req = convert::intersect_req(request.into_inner())?;
        let api_res = self
            .inner
            .intersect(api_req)
            .await
            .map_err(convert::service_error)?;
        Ok(Response::new(convert::intersect_res(api_res)))
    }

    async fn accessibility(
        &self,
        request: Request<AccessibilityRequest>,
    ) -> Result<Response<AccessibilityResponse>, Status> {
        let api_req = convert::accessibility_req(request.into_inner());
        let api_res = self
            .inner
            .accessibility(api_req)
            .await
            .map_err(convert::service_error)?;
        Ok(Response::new(convert::accessibility_res(api_res)))
    }
}

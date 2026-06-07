use horizon_api::{
    ApiCoordinate, ApiLineString, CalculateHorizonAccessRequest as ApiCalculateHorizonAccessRequest,
    CalculateHorizonAccessResponse as ApiCalculateHorizonAccessResponse, ServiceError,
};
use horizon_proto::{
    CalculateHorizonAccessRequest, CalculateHorizonAccessResponse, Coordinate, LineString,
};
use tonic::Status;

pub fn horizon_access_req(
    proto: CalculateHorizonAccessRequest,
) -> Result<ApiCalculateHorizonAccessRequest, Status> {
    let viewpoint = proto
        .viewpoint
        .ok_or_else(|| Status::invalid_argument("viewpoint is required"))?;
    let target_coastline = proto
        .target_coastline
        .ok_or_else(|| Status::invalid_argument("target_coastline is required"))?;

    let coastline = line_string(target_coastline)?;
    if coastline.points.len() < 2 {
        return Err(Status::invalid_argument(
            "target_coastline must contain at least two points",
        ));
    }

    Ok(ApiCalculateHorizonAccessRequest {
        viewpoint: coordinate(viewpoint),
        target_coastline: coastline,
    })
}

pub fn horizon_access_res(
    api: ApiCalculateHorizonAccessResponse,
) -> CalculateHorizonAccessResponse {
    CalculateHorizonAccessResponse {
        obstruction_percentage: api.obstruction_percentage,
        rays_cast: api.rays_cast,
        rays_obstructed: api.rays_obstructed,
    }
}

fn coordinate(proto: Coordinate) -> ApiCoordinate {
    ApiCoordinate {
        x: proto.x,
        y: proto.y,
        z: proto.z,
    }
}

fn line_string(proto: LineString) -> Result<ApiLineString, Status> {
    if proto.points.is_empty() {
        return Err(Status::invalid_argument(
            "target_coastline must not be empty",
        ));
    }

    Ok(ApiLineString {
        points: proto.points.into_iter().map(coordinate).collect(),
    })
}

pub fn service_error(err: ServiceError) -> Status {
    match err {
        ServiceError::NotLoaded => Status::failed_precondition(err.to_string()),
        ServiceError::OutOfBounds => Status::out_of_range(err.to_string()),
        ServiceError::InvalidRequest(_) => Status::invalid_argument(err.to_string()),
        ServiceError::LoadFailed(_) => Status::internal(err.to_string()),
        ServiceError::Compute(_) => Status::internal(err.to_string()),
        ServiceError::Internal(_) => Status::internal(err.to_string()),
    }
}

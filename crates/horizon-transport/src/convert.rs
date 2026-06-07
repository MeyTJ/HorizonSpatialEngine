use horizon_api::{
    AccessibilityRequest as ApiAccessibilityRequest,
    AccessibilityResponse as ApiAccessibilityResponse, BoundingBox as ApiBoundingBox,
    IntersectRequest as ApiIntersectRequest, IntersectResponse as ApiIntersectResponse,
    LoadDatasetRequest as ApiLoadDatasetRequest, LoadDatasetResponse as ApiLoadDatasetResponse,
    ServiceError,
};
use horizon_proto::{
    AccessibilityRequest, AccessibilityResponse, BoundingBox, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse,
};
use tonic::Status;

pub fn load_dataset_req(proto: LoadDatasetRequest) -> ApiLoadDatasetRequest {
    ApiLoadDatasetRequest { path: proto.path }
}

pub fn load_dataset_res(api: ApiLoadDatasetResponse) -> LoadDatasetResponse {
    LoadDatasetResponse {
        building_count: api.building_count,
        path: api.path,
    }
}

pub fn intersect_req(proto: IntersectRequest) -> Result<ApiIntersectRequest, Status> {
    let bounds = proto
        .bounds
        .ok_or_else(|| Status::invalid_argument("bounds required"))?;
    Ok(ApiIntersectRequest {
        bounds: bounding_box(bounds),
    })
}

pub fn intersect_res(api: ApiIntersectResponse) -> IntersectResponse {
    IntersectResponse {
        building_ids: api.building_ids,
        matched_count: api.matched_count,
        query_area: api.query_area,
    }
}

pub fn accessibility_req(proto: AccessibilityRequest) -> ApiAccessibilityRequest {
    ApiAccessibilityRequest {
        observer_x: proto.observer_x,
        observer_y: proto.observer_y,
        observer_z: proto.observer_z,
        radius: proto.radius,
    }
}

pub fn accessibility_res(api: ApiAccessibilityResponse) -> AccessibilityResponse {
    AccessibilityResponse {
        visible_building_count: api.visible_building_count,
        skyline_obstruction_ratio: api.skyline_obstruction_ratio,
        mean_view_distance: api.mean_view_distance,
    }
}

fn bounding_box(proto: BoundingBox) -> ApiBoundingBox {
    ApiBoundingBox {
        min_x: proto.min_x,
        min_y: proto.min_y,
        max_x: proto.max_x,
        max_y: proto.max_y,
        min_z: proto.min_z,
        max_z: proto.max_z,
    }
}

pub fn service_error(err: ServiceError) -> Status {
    match err {
        ServiceError::NotLoaded => Status::failed_precondition(err.to_string()),
        ServiceError::OutOfBounds => Status::out_of_range(err.to_string()),
        ServiceError::InvalidRequest(_) => Status::invalid_argument(err.to_string()),
        ServiceError::LoadFailed(_) | ServiceError::Compute(_) => {
            Status::internal(err.to_string())
        }
        ServiceError::Internal(_) => Status::internal(err.to_string()),
    }
}

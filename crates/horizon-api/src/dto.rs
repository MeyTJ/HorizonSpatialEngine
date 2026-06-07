use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("dataset not loaded")]
    NotLoaded,

    #[error("failed to load dataset: {0}")]
    LoadFailed(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("compute error: {0}")]
    Compute(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("coordinates out of dataset bounds")]
    OutOfBounds,
}

#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone)]
pub struct LoadDatasetRequest {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct LoadDatasetResponse {
    pub building_count: u64,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct IntersectRequest {
    pub bounds: BoundingBox,
}

#[derive(Debug, Clone)]
pub struct IntersectResponse {
    pub building_ids: Vec<u64>,
    pub matched_count: u64,
    pub query_area: f64,
}

#[derive(Debug, Clone)]
pub struct AccessibilityRequest {
    pub observer_x: f64,
    pub observer_y: f64,
    pub observer_z: f64,
    pub radius: f64,
}

#[derive(Debug, Clone)]
pub struct AccessibilityResponse {
    pub visible_building_count: u64,
    pub skyline_obstruction_ratio: f64,
    pub mean_view_distance: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct ApiCoordinate {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
pub struct ApiLineString {
    pub points: Vec<ApiCoordinate>,
}

#[derive(Debug, Clone)]
pub struct CalculateHorizonAccessRequest {
    pub viewpoint: ApiCoordinate,
    pub target_coastline: ApiLineString,
}

#[derive(Debug, Clone, Copy)]
pub struct CalculateHorizonAccessResponse {
    pub obstruction_percentage: f64,
    pub rays_cast: u32,
    pub rays_obstructed: u32,
}

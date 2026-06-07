use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use horizon_api::{
    AccessibilityRequest, AccessibilityResponse, ApiCoordinate, CalculateHorizonAccessRequest,
    CalculateHorizonAccessResponse, IntersectRequest, IntersectResponse, LoadDatasetRequest,
    LoadDatasetResponse, ServiceError, SpatialComputeService, SpatialService,
};
use horizon_core::{
    calculate_horizon_blockage, CoreError, QueryBounds, SharedSpatialIndex, SpatialEngine,
    SpatialIndex, SpatialQuery,
};
use horizon_geometry::{Coordinate, LineString};
use tracing::instrument;

/// Bridges the compute core to the transport-agnostic API boundary.
pub struct CoreAdapter {
    index: RwLock<Option<SharedSpatialIndex>>,
}

impl CoreAdapter {
    pub fn new() -> Self {
        Self {
            index: RwLock::new(None),
        }
    }

    /// Clone the shared index handle and release the lock before querying.
    fn snapshot(&self) -> Result<SharedSpatialIndex, ServiceError> {
        let guard = self
            .index
            .read()
            .map_err(|_| ServiceError::Internal("index lock poisoned".into()))?;

        guard
            .as_ref()
            .map(Arc::clone)
            .ok_or(ServiceError::NotLoaded)
    }
}

impl Default for CoreAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn map_core_error(err: CoreError) -> ServiceError {
    match err {
        CoreError::OutOfBounds => ServiceError::OutOfBounds,
        CoreError::InvalidCoastline(msg) => ServiceError::InvalidRequest(msg),
        CoreError::NoDataset => ServiceError::NotLoaded,
        CoreError::Storage(err) => ServiceError::LoadFailed(err.to_string()),
        CoreError::IndexBuild(msg) | CoreError::HorizonAnalysis(msg) => {
            ServiceError::Compute(msg)
        }
    }
}

fn to_coordinate(point: ApiCoordinate) -> Coordinate {
    Coordinate::new(point.x, point.y, point.z)
}

fn to_line_string(line: horizon_api::ApiLineString) -> LineString {
    LineString::new(line.points.into_iter().map(to_coordinate).collect())
}

#[async_trait]
impl SpatialService for CoreAdapter {
    #[instrument(skip(self))]
    async fn load_dataset(
        &self,
        request: LoadDatasetRequest,
    ) -> Result<LoadDatasetResponse, ServiceError> {
        let path = PathBuf::from(&request.path);
        if request.path.is_empty() {
            return Err(ServiceError::InvalidRequest("path must not be empty".into()));
        }

        let index =
            SpatialIndex::open(&path).map_err(|e| ServiceError::LoadFailed(e.to_string()))?;

        let response = LoadDatasetResponse {
            building_count: index.feature_count() as u64,
            path: request.path,
        };

        let mut guard = self
            .index
            .write()
            .map_err(|_| ServiceError::Internal("index lock poisoned".into()))?;
        *guard = Some(index);

        Ok(response)
    }

    #[instrument(skip(self))]
    async fn intersect(
        &self,
        request: IntersectRequest,
    ) -> Result<IntersectResponse, ServiceError> {
        let bounds = QueryBounds::new(
            request.bounds.min_x,
            request.bounds.min_y,
            request.bounds.max_x,
            request.bounds.max_y,
        )
        .with_elevation(request.bounds.min_z, request.bounds.max_z);

        let index = self.snapshot()?;
        let engine = SpatialEngine::from_index(index);
        let result = engine
            .execute(SpatialQuery::Intersect(bounds))
            .map_err(|e| ServiceError::Compute(e.to_string()))?;

        match result {
            horizon_core::QueryResult::Intersection(r) => Ok(IntersectResponse {
                building_ids: r.building_ids,
                matched_count: r.matched_count as u64,
                query_area: r.query_area,
            }),
            _ => Err(ServiceError::Internal("unexpected query result".into())),
        }
    }

    #[instrument(skip(self))]
    async fn accessibility(
        &self,
        request: AccessibilityRequest,
    ) -> Result<AccessibilityResponse, ServiceError> {
        if request.radius <= 0.0 {
            return Err(ServiceError::InvalidRequest("radius must be positive".into()));
        }

        let index = self.snapshot()?;
        let engine = SpatialEngine::from_index(index);
        let result = engine
            .execute(SpatialQuery::Accessibility {
                observer_x: request.observer_x,
                observer_y: request.observer_y,
                observer_z: request.observer_z,
                radius: request.radius,
            })
            .map_err(|e| ServiceError::Compute(e.to_string()))?;

        match result {
            horizon_core::QueryResult::Accessibility(r) => Ok(AccessibilityResponse {
                visible_building_count: r.visible_building_count as u64,
                skyline_obstruction_ratio: r.skyline_obstruction_ratio,
                mean_view_distance: r.mean_view_distance,
            }),
            _ => Err(ServiceError::Internal("unexpected query result".into())),
        }
    }
}

#[async_trait]
impl SpatialComputeService for CoreAdapter {
    #[instrument(skip(self, request))]
    async fn calculate_horizon_access(
        &self,
        request: CalculateHorizonAccessRequest,
    ) -> Result<CalculateHorizonAccessResponse, ServiceError> {
        let index = self.snapshot()?;
        let viewpoint = to_coordinate(request.viewpoint);
        let coastline = to_line_string(request.target_coastline);

        index
            .validate_in_dataset_bounds(viewpoint)
            .map_err(map_core_error)?;
        index
            .validate_coastline_in_bounds(&coastline)
            .map_err(map_core_error)?;

        let result =
            calculate_horizon_blockage(&index, viewpoint, &coastline).map_err(map_core_error)?;

        Ok(CalculateHorizonAccessResponse {
            obstruction_percentage: result.obstruction_percentage,
            rays_cast: result.rays_cast as u32,
            rays_obstructed: result.rays_obstructed as u32,
        })
    }
}

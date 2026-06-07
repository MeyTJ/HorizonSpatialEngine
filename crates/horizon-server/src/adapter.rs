use std::path::PathBuf;
use std::sync::RwLock;

use async_trait::async_trait;
use horizon_api::{
    AccessibilityRequest, AccessibilityResponse, IntersectRequest, IntersectResponse,
    LoadDatasetRequest, LoadDatasetResponse, ServiceError, SpatialService,
};
use horizon_core::{QueryBounds, SpatialEngine, SpatialQuery};

/// Bridges the compute core to the transport-agnostic API boundary.
pub struct CoreAdapter {
    engine: RwLock<Option<SpatialEngine>>,
}

impl CoreAdapter {
    pub fn new() -> Self {
        Self {
            engine: RwLock::new(None),
        }
    }

    fn with_engine<F, T>(&self, f: F) -> Result<T, ServiceError>
    where
        F: FnOnce(&SpatialEngine) -> Result<T, ServiceError>,
    {
        let guard = self
            .engine
            .read()
            .map_err(|_| ServiceError::Internal("engine lock poisoned".into()))?;

        let engine = guard.as_ref().ok_or(ServiceError::NotLoaded)?;
        f(engine)
    }
}

impl Default for CoreAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpatialService for CoreAdapter {
    async fn load_dataset(
        &self,
        request: LoadDatasetRequest,
    ) -> Result<LoadDatasetResponse, ServiceError> {
        let path = PathBuf::from(&request.path);
        if request.path.is_empty() {
            return Err(ServiceError::InvalidRequest("path must not be empty".into()));
        }

        let engine = SpatialEngine::open(&path).map_err(|e| ServiceError::LoadFailed(e.to_string()))?;

        let response = LoadDatasetResponse {
            building_count: engine.building_count() as u64,
            path: request.path,
        };

        let mut guard = self
            .engine
            .write()
            .map_err(|_| ServiceError::Internal("engine lock poisoned".into()))?;
        *guard = Some(engine);

        Ok(response)
    }

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

        self.with_engine(|engine| {
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
        })
    }

    async fn accessibility(
        &self,
        request: AccessibilityRequest,
    ) -> Result<AccessibilityResponse, ServiceError> {
        if request.radius <= 0.0 {
            return Err(ServiceError::InvalidRequest("radius must be positive".into()));
        }

        self.with_engine(|engine| {
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
        })
    }
}

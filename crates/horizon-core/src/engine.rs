use std::path::Path;

use horizon_storage::{MappedDataset, MmapLoader};
use tracing::{info, instrument};

use crate::error::CoreError;
use crate::query::{AccessibilityResult, IntersectionResult, QueryBounds, SpatialQuery};
use crate::spatial::BuildingIndex;

/// Primary compute engine — owns the memory-mapped dataset and spatial index.
pub struct SpatialEngine {
    dataset: MappedDataset,
    index: BuildingIndex,
}

impl SpatialEngine {
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let dataset = MmapLoader::load(path)?;
        let buildings = dataset.archived().buildings.as_slice();
        let index = BuildingIndex::build(buildings);

        info!(
            buildings = index.building_count(),
            path = %dataset.path().display(),
            "dataset loaded via memory-map"
        );

        Ok(Self { dataset, index })
    }

    pub fn building_count(&self) -> usize {
        self.index.building_count()
    }

    pub fn dataset_path(&self) -> &Path {
        self.dataset.path()
    }

    pub fn execute(&self, query: SpatialQuery) -> Result<QueryResult, CoreError> {
        match query {
            SpatialQuery::Intersect(bounds) => {
                Ok(QueryResult::Intersection(self.intersect(bounds)?))
            }
            SpatialQuery::Accessibility {
                observer_x,
                observer_y,
                observer_z,
                radius,
            } => Ok(QueryResult::Accessibility(self.accessibility(
                observer_x,
                observer_y,
                observer_z,
                radius,
            )?)),
        }
    }

    fn intersect(&self, bounds: QueryBounds) -> Result<IntersectionResult, CoreError> {
        let indices = self.index.query_intersects(&bounds);
        let buildings = self.dataset.archived().buildings.as_slice();

        let building_ids: Vec<u64> = indices
            .iter()
            .filter_map(|&idx| buildings.get(idx).map(|b| b.id))
            .collect();

        Ok(IntersectionResult {
            matched_count: building_ids.len(),
            building_ids,
            query_area: bounds.area(),
        })
    }

    fn accessibility(
        &self,
        observer_x: f64,
        observer_y: f64,
        observer_z: f64,
        radius: f64,
    ) -> Result<AccessibilityResult, CoreError> {
        let bounds = QueryBounds::new(
            observer_x - radius,
            observer_y - radius,
            observer_x + radius,
            observer_y + radius,
        )
        .with_elevation(observer_z, observer_z + 500.0);

        let indices = self.index.query_intersects(&bounds);
        let buildings = self.dataset.archived().buildings.as_slice();

        let mut visible = 0usize;
        let mut total_distance = 0.0f64;
        let mut obstructed = 0usize;

        for &idx in &indices {
            let Some(building) = buildings.get(idx) else {
                continue;
            };

            let cx = centroid_x(building);
            let cy = centroid_y(building);
            let dx = cx - observer_x;
            let dy = cy - observer_y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius {
                continue;
            }

            visible += 1;
            total_distance += dist;

            if building.height > observer_z {
                obstructed += 1;
            }
        }

        let skyline_obstruction_ratio = if visible == 0 {
            0.0
        } else {
            obstructed as f64 / visible as f64
        };

        let mean_view_distance = if visible == 0 {
            0.0
        } else {
            total_distance / visible as f64
        };

        Ok(AccessibilityResult {
            visible_building_count: visible,
            skyline_obstruction_ratio,
            mean_view_distance,
        })
    }
}

fn centroid_x(building: &horizon_geometry::ArchivedBuilding) -> f64 {
    let verts = building.footprint.vertices.as_slice();
    if verts.is_empty() {
        return 0.0;
    }
    verts.iter().map(|v| v.x).sum::<f64>() / verts.len() as f64
}

fn centroid_y(building: &horizon_geometry::ArchivedBuilding) -> f64 {
    let verts = building.footprint.vertices.as_slice();
    if verts.is_empty() {
        return 0.0;
    }
    verts.iter().map(|v| v.y).sum::<f64>() / verts.len() as f64
}

/// Discriminated query result returned by the compute engine.
#[derive(Debug, Clone)]
pub enum QueryResult {
    Intersection(IntersectionResult),
    Accessibility(AccessibilityResult),
}

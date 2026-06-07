use std::path::Path;
use std::sync::Arc;

use horizon_geometry::ArchivedBuilding;
use horizon_storage::{MappedDataset, MmapLoader};
use rayon::prelude::*;
use rstar::{AABB, RTree};
use tracing::{info, instrument};

use crate::error::CoreError;
use crate::query::QueryBounds;
use crate::spatial::SpatialEntry;

/// Thread-safe handle shared across gRPC worker threads.
pub type SharedSpatialIndex = Arc<SpatialIndex>;

/// In-memory R-tree over memory-mapped rkyv urban geometry.
///
/// Polygon coordinates remain on disk in the mapped archive; the tree stores
/// only compact envelopes and feature indices. `RTree` queries are read-only,
/// so concurrent access via `Arc<SpatialIndex>` requires no locks.
pub struct SpatialIndex {
    dataset: Arc<MappedDataset>,
    tree: RTree<SpatialEntry>,
    feature_count: usize,
}

impl SpatialIndex {
    /// Memory-map an rkyv archive and bulk-load the spatial index.
    #[instrument(skip_all, fields(path = %path.as_ref().display()))]
    pub fn open(path: impl AsRef<Path>) -> Result<SharedSpatialIndex, CoreError> {
        let dataset = MmapLoader::load(path)?;
        Ok(Self::from_mapped(dataset))
    }

    /// Build an index from an already-mapped dataset.
    #[instrument(skip(dataset), fields(path = %dataset.path().display()))]
    pub fn from_mapped(dataset: MappedDataset) -> SharedSpatialIndex {
        let path_display = dataset.path().display().to_string();
        let buildings = dataset.archived().buildings.as_slice();

        let entries: Vec<SpatialEntry> = buildings
            .par_iter()
            .enumerate()
            .filter_map(|(idx, building)| SpatialEntry::from_archived(idx, building))
            .collect();

        let feature_count = entries.len();
        let tree = RTree::bulk_load(entries);

        info!(
            features = feature_count,
            path = %path_display,
            "spatial index bulk-loaded from memory-mapped archive"
        );

        Arc::new(Self {
            dataset: Arc::new(dataset),
            tree,
            feature_count,
        })
    }

    pub fn feature_count(&self) -> usize {
        self.feature_count
    }

    pub fn dataset_path(&self) -> &Path {
        self.dataset.path()
    }

    /// Zero-copy slice into the memory-mapped building archive.
    #[inline]
    pub fn buildings(&self) -> &[ArchivedBuilding] {
        self.dataset.archived().buildings.as_slice()
    }

    /// Zero-copy reference to a single archived building by index.
    #[inline]
    pub fn building(&self, feature_idx: usize) -> Option<&ArchivedBuilding> {
        self.buildings().get(feature_idx)
    }

    /// Validate that a coordinate lies within the dataset header bounds.
    #[instrument(skip(self), fields(x = coord.x, y = coord.y, z = coord.z))]
    pub fn validate_in_dataset_bounds(
        &self,
        coord: horizon_geometry::Coordinate,
    ) -> Result<(), CoreError> {
        let header = &self.dataset.archived().header;
        if coord.x < header.bounds_min.x
            || coord.x > header.bounds_max.x
            || coord.y < header.bounds_min.y
            || coord.y > header.bounds_max.y
        {
            return Err(CoreError::OutOfBounds);
        }
        Ok(())
    }

    /// Validate all coastline vertices lie within dataset bounds.
    #[instrument(skip(self, coastline))]
    pub fn validate_coastline_in_bounds(
        &self,
        coastline: &horizon_geometry::LineString,
    ) -> Result<(), CoreError> {
        for point in &coastline.points {
            self.validate_in_dataset_bounds(*point)?;
        }
        Ok(())
    }

    /// Return indices of features whose XY envelope intersects `bounds`.
    #[instrument(skip(self), fields(
        min_x = bounds.min_x,
        min_y = bounds.min_y,
        max_x = bounds.max_x,
        max_y = bounds.max_y
    ))]
    pub fn query_intersects(&self, bounds: &QueryBounds) -> Vec<usize> {
        let search =
            AABB::from_corners([bounds.min_x, bounds.min_y], [bounds.max_x, bounds.max_y]);

        self.tree
            .locate_in_envelope_intersecting(&search)
            .filter(|entry| entry.intersects_z(bounds.min_z, bounds.max_z))
            .map(|entry| entry.feature_idx)
            .collect()
    }

    /// Return building IDs for features intersecting `bounds`.
    #[instrument(skip(self))]
    pub fn query_building_ids(&self, bounds: &QueryBounds) -> Vec<u64> {
        let buildings = self.buildings();
        self.query_intersects(bounds)
            .into_iter()
            .filter_map(|idx| buildings.get(idx).map(|b| b.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_geometry::{
        Building, Point3, Polygon, UrbanDataset, UrbanDatasetHeader, DATASET_MAGIC, DATASET_VERSION,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_test_archive(buildings: Vec<Building>) -> NamedTempFile {
        let mut bounds_min = Point3::new(f64::MAX, f64::MAX, 0.0);
        let mut bounds_max = Point3::new(f64::MIN, f64::MIN, 0.0);
        for building in &buildings {
            for v in &building.footprint.vertices {
                bounds_min.x = bounds_min.x.min(v.x);
                bounds_min.y = bounds_min.y.min(v.y);
                bounds_max.x = bounds_max.x.max(v.x);
                bounds_max.y = bounds_max.y.max(v.y);
            }
        }
        let header = UrbanDatasetHeader {
            magic: DATASET_MAGIC,
            version: DATASET_VERSION,
            building_count: buildings.len() as u64,
            bounds_min,
            bounds_max,
            crs_epsg: 4326,
        };
        let dataset = UrbanDataset::new(header, buildings);
        let bytes = MmapLoader::serialize(&dataset);
        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(bytes.as_slice()).expect("write archive");
        file
    }

    #[test]
    fn query_finds_intersecting_archived_features() {
        let b1 = Building::new(
            100,
            Polygon::new(
                vec![
                    Point3::new(0.0, 0.0, 0.0),
                    Point3::new(2.0, 0.0, 0.0),
                    Point3::new(2.0, 2.0, 0.0),
                    Point3::new(0.0, 2.0, 0.0),
                ],
                0.0,
                5.0,
            ),
            5.0,
            1,
            1,
        );
        let b2 = Building::new(
            200,
            Polygon::new(
                vec![
                    Point3::new(10.0, 10.0, 0.0),
                    Point3::new(12.0, 10.0, 0.0),
                    Point3::new(12.0, 12.0, 0.0),
                    Point3::new(10.0, 12.0, 0.0),
                ],
                0.0,
                5.0,
            ),
            5.0,
            1,
            1,
        );

        let file = write_test_archive(vec![b1, b2]);
        let index = SpatialIndex::open(file.path()).expect("open index");
        let hits = index.query_intersects(&QueryBounds::new(1.0, 1.0, 3.0, 3.0));
        assert_eq!(hits.len(), 1);
        assert_eq!(index.building(hits[0]).expect("building").id, 100);
    }
}

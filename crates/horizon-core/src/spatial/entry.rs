use horizon_geometry::ArchivedBuilding;
use rstar::{AABB, RTreeObject};

/// Leaf node stored in the R-tree — fixed-size envelope plus an index into the
/// memory-mapped archive. No polygon coordinates are copied into the tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialEntry {
    pub feature_idx: usize,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl SpatialEntry {
    /// Compute an axis-aligned envelope by scanning archived vertices in place.
    pub fn from_archived(feature_idx: usize, building: &ArchivedBuilding) -> Option<Self> {
        let verts = building.footprint.vertices.as_slice();
        if verts.is_empty() {
            return None;
        }

        let (mut min_x, mut min_y) = (verts[0].x, verts[0].y);
        let (mut max_x, mut max_y) = (min_x, min_y);

        for vertex in &verts[1..] {
            min_x = min_x.min(vertex.x);
            min_y = min_y.min(vertex.y);
            max_x = max_x.max(vertex.x);
            max_y = max_y.max(vertex.y);
        }

        let min_z = building.footprint.elevation_min;
        let max_z = building.footprint.elevation_max.max(building.height);

        Some(Self {
            feature_idx,
            min_x,
            min_y,
            max_x,
            max_y,
            min_z,
            max_z,
        })
    }

    pub fn intersects_xy(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> bool {
        self.min_x <= max_x && self.max_x >= min_x && self.min_y <= max_y && self.max_y >= min_y
    }

    pub fn intersects_z(&self, min_z: f64, max_z: f64) -> bool {
        self.min_z <= max_z && self.max_z >= min_z
    }
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_geometry::{Building, Point3, Polygon, UrbanDataset, UrbanDatasetHeader};
    use rkyv::util::AlignedVec;
    use rkyv::{Archive, Deserialize, Serialize};

    fn archived_building(id: u64, x: f64, y: f64) -> AlignedVec {
        let footprint = Polygon::new(
            vec![
                Point3::new(x, y, 0.0),
                Point3::new(x + 1.0, y, 0.0),
                Point3::new(x + 1.0, y + 1.0, 0.0),
                Point3::new(x, y + 1.0, 0.0),
            ],
            0.0,
            10.0,
        );
        let building = Building::new(id, footprint, 10.0, 3, 1);
        let header = UrbanDatasetHeader::new(
            1,
            Point3::new(x, y, 0.0),
            Point3::new(x + 1.0, y + 1.0, 10.0),
            4326,
        );
        let dataset = UrbanDataset::new(header, vec![building]);
        rkyv::to_bytes::<rkyv::rancor::Error>(&dataset).expect("test serialize")
    }

    #[test]
    fn envelope_from_archived_vertices() {
        let bytes = archived_building(1, 52.0, 36.0);
        let archived = rkyv::access_unchecked::<horizon_geometry::ArchivedUrbanDataset>(&bytes);
        let building = archived.buildings.as_slice().first().expect("building");
        let entry = SpatialEntry::from_archived(0, building).expect("entry");
        assert_eq!(entry.min_x, 52.0);
        assert_eq!(entry.max_x, 53.0);
        assert_eq!(entry.max_z, 10.0);
    }
}

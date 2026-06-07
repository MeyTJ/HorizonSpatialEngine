use horizon_geometry::ArchivedBuilding;
use rstar::{AABB, RTree, RTreeObject};

/// Axis-aligned bounding envelope for spatial indexing.
#[derive(Debug, Clone, Copy)]
pub struct BuildingEnvelope {
    pub building_idx: usize,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl RTreeObject for BuildingEnvelope {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}

/// Lock-free read-optimized R-tree over building footprints.
pub struct BuildingIndex {
    tree: RTree<BuildingEnvelope>,
    building_count: usize,
}

impl BuildingIndex {
    pub fn build(buildings: &[ArchivedBuilding]) -> Self {
        let envelopes: Vec<BuildingEnvelope> = buildings
            .iter()
            .enumerate()
            .filter_map(|(idx, b)| footprint_envelope(idx, b))
            .collect();

        let building_count = envelopes.len();
        let tree = RTree::bulk_load(envelopes);

        Self {
            tree,
            building_count,
        }
    }

    pub fn building_count(&self) -> usize {
        self.building_count
    }

    pub fn query_intersects(&self, bounds: &QueryBounds) -> Vec<usize> {
        let search = AABB::from_corners([bounds.min_x, bounds.min_y], [bounds.max_x, bounds.max_y]);
        self.tree
            .locate_in_envelope_intersecting(&search)
            .map(|e| e.building_idx)
            .collect()
    }
}

use crate::query::QueryBounds;

fn footprint_envelope(idx: usize, building: &ArchivedBuilding) -> Option<BuildingEnvelope> {
    let verts = building.footprint.vertices.as_slice();
    if verts.is_empty() {
        return None;
    }

    let (mut min_x, mut min_y) = (verts[0].x, verts[0].y);
    let (mut max_x, mut max_y) = (min_x, min_y);

    for v in &verts[1..] {
        min_x = min_x.min(v.x);
        min_y = min_y.min(v.y);
        max_x = max_x.max(v.x);
        max_y = max_y.max(v.y);
    }

    Some(BuildingEnvelope {
        building_idx: idx,
        min_x,
        min_y,
        max_x,
        max_y,
    })
}

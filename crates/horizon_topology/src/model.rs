use horizon_geometry::Polygon;

/// Axis-aligned 3D bounding box derived from footprint XY extent and elevation range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox3D {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl BoundingBox3D {
    pub fn volume(&self) -> f64 {
        (self.max_x - self.min_x).abs()
            * (self.max_y - self.min_y).abs()
            * (self.max_z - self.min_z).abs()
    }
}

/// A single urban topology feature loaded from PostGIS.
#[derive(Debug, Clone)]
pub struct UrbanTopologyFeature {
    pub id: i64,
    pub neighborhood_name: String,
    pub bbox: BoundingBox3D,
    pub footprint: Polygon,
}

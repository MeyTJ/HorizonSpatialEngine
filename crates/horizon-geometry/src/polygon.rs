use crate::Point3;
use rkyv::{Archive, Deserialize, Serialize};

/// A closed polygon ring defined by an ordered vertex sequence.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct Polygon {
    pub vertices: Vec<Point3>,
    pub elevation_min: f64,
    pub elevation_max: f64,
}

impl Polygon {
    pub fn new(vertices: Vec<Point3>, elevation_min: f64, elevation_max: f64) -> Self {
        Self {
            vertices,
            elevation_min,
            elevation_max,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

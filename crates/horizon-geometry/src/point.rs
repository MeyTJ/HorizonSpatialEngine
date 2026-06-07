use rkyv::{Archive, Deserialize, Serialize};

/// A 3D point in WGS84/ECEF or local projected coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

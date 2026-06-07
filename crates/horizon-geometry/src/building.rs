use crate::Polygon;
use rkyv::{Archive, Deserialize, Serialize};

/// An urban building footprint with extruded volume metadata.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct Building {
    pub id: u64,
    pub footprint: Polygon,
    pub height: f64,
    pub floor_count: u32,
    pub land_use_code: u16,
}

impl Building {
    pub fn new(
        id: u64,
        footprint: Polygon,
        height: f64,
        floor_count: u32,
        land_use_code: u16,
    ) -> Self {
        Self {
            id,
            footprint,
            height,
            floor_count,
            land_use_code,
        }
    }
}

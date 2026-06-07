use crate::{Building, DATASET_MAGIC, DATASET_VERSION, Point3};
use rkyv::{Archive, Deserialize, Serialize};

/// Fixed-size header prepended to every memory-mapped dataset file.
#[derive(Debug, Clone, Copy, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct UrbanDatasetHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub building_count: u64,
    pub bounds_min: Point3,
    pub bounds_max: Point3,
    pub crs_epsg: u32,
}

impl UrbanDatasetHeader {
    pub fn new(
        building_count: u64,
        bounds_min: Point3,
        bounds_max: Point3,
        crs_epsg: u32,
    ) -> Self {
        Self {
            magic: DATASET_MAGIC,
            version: DATASET_VERSION,
            building_count,
            bounds_min,
            bounds_max,
            crs_epsg,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == DATASET_MAGIC && self.version == DATASET_VERSION
    }
}

/// Root archive structure for an entire urban geometry dataset.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct UrbanDataset {
    pub header: UrbanDatasetHeader,
    pub buildings: Vec<Building>,
}

impl UrbanDataset {
    pub fn new(header: UrbanDatasetHeader, buildings: Vec<Building>) -> Self {
        Self { header, buildings }
    }

    pub fn building_count(&self) -> usize {
        self.buildings.len()
    }
}

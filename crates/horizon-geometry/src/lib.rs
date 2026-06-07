//! Zero-copy urban geometry primitives archived with rkyv.
//!
//! All types in this crate are designed for direct access from memory-mapped
//! buffers without heap allocation or deserialization copies.

pub mod building;
pub mod dataset;
pub mod point;
pub mod polygon;

pub use building::{ArchivedBuilding, Building};
pub use dataset::{ArchivedUrbanDataset, UrbanDataset, UrbanDatasetHeader};
pub use point::{ArchivedPoint3, Point3};
pub use polygon::{ArchivedPolygon, Polygon};

/// Magic bytes identifying a valid Horizon urban geometry archive.
pub const DATASET_MAGIC: [u8; 4] = *b"HZRN";

/// Current on-disk archive format version.
pub const DATASET_VERSION: u32 = 1;

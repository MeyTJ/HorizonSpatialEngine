//! R-tree spatial index backed by zero-copy rkyv archived geometry.

mod entry;
mod index;

pub use entry::SpatialEntry;
pub use index::{SharedSpatialIndex, SpatialIndex};

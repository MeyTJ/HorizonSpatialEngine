//! Memory-mapped zero-copy dataset ingestion.
//!
//! Datasets are accessed directly from the mapped file region via rkyv archived
//! types — no intermediate heap copies of geometry data are performed.

mod error;
mod mmap;

pub use error::StorageError;
pub use mmap::{MappedDataset, MmapLoader};

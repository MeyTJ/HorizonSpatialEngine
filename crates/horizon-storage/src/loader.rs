use std::path::{Path, PathBuf};
use std::sync::Arc;

use horizon_geometry::{ArchivedUrbanDataset, UrbanDatasetHeader, DATASET_MAGIC};
use memmap2::Mmap;
use rkyv::access::Archived;

use crate::error::StorageError;
use crate::mapped::MappedDataset;

/// Loads urban geometry datasets from memory-mapped files.
pub struct DatasetLoader {
    validate: bool,
}

impl Default for DatasetLoader {
    fn default() -> Self {
        Self { validate: true }
    }
}

impl DatasetLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable bytecheck validation for trusted local datasets.
    pub fn without_validation(mut self) -> Self {
        self.validate = false;
        self
    }

    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<MappedDataset, StorageError> {
        let path = path.as_ref();
        let path_str = path.display().to_string();

        let file = std::fs::File::open(path).map_err(|source| StorageError::Open {
            path: path_str.clone(),
            source,
        })?;

        let mmap = unsafe {
            Mmap::map(&file).map_err(|source| StorageError::Map {
                path: path_str.clone(),
                source,
            })?
        };

        if mmap.len() < std::mem::size_of::<UrbanDatasetHeader>() {
            return Err(StorageError::Empty { path: path_str });
        }

        if &mmap[0..4] != DATASET_MAGIC {
            return Err(StorageError::InvalidHeader { path: path_str });
        }

        let archived = access_archived(&mmap, &path_str, self.validate)?;

        Ok(MappedDataset {
            path: PathBuf::from(path),
            mmap: Arc::new(mmap),
            archived,
        })
    }
}

fn access_archived<'a>(
    bytes: &'a [u8],
    path: &str,
    validate: bool,
) -> Result<&'a Archived<UrbanDataset>, StorageError> {
    if validate {
        rkyv::access::<ArchivedUrbanDataset, rkyv::rancor::Error>(bytes).map_err(|e| {
            StorageError::ValidationFailed {
                path: path.to_owned(),
                detail: e.to_string(),
            }
        })
    } else {
        rkyv::access_unchecked::<ArchivedUrbanDataset>(bytes).map_err(|e| {
            StorageError::ArchiveAccess {
                path: path.to_owned(),
                detail: e.to_string(),
            }
        })
    }
}

use horizon_geometry::UrbanDataset;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use horizon_geometry::{ArchivedUrbanDataset, UrbanDataset};
use memmap2::Mmap;
use rkyv::access::Archived;
use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize};

use crate::StorageError;

/// Loads urban geometry datasets from memory-mapped files.
pub struct MmapLoader;

impl MmapLoader {
    /// Memory-map `path` and return a zero-copy view into the archived dataset.
    pub fn load(path: impl AsRef<Path>) -> Result<MappedDataset, StorageError> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|source| StorageError::Open {
            path: path.clone(),
            source,
        })?;

        let mmap = unsafe {
            Mmap::map(&file).map_err(|source| StorageError::Map {
                path: path.clone(),
                source,
            })?
        };

        if mmap.is_empty() {
            return Err(StorageError::Empty { path });
        }

        let archived = access_archived(&mmap, &path)?;

        Ok(MappedDataset {
            path,
            _mmap: Arc::new(mmap),
            archived,
        })
    }

    /// Serialize a dataset to an aligned byte buffer suitable for writing to disk.
    pub fn serialize(dataset: &UrbanDataset) -> AlignedVec {
        rkyv::to_bytes::<rkyv::rancor::Error>(dataset)
            .expect("in-memory serialization of UrbanDataset must succeed")
    }
}

/// A memory-mapped dataset providing zero-copy access to archived geometry.
pub struct MappedDataset {
    path: PathBuf,
    _mmap: Arc<Mmap>,
    archived: *const ArchivedUrbanDataset,
}

// Safety: `_mmap` keeps the backing bytes alive for the lifetime of this handle.
unsafe impl Send for MappedDataset {}
unsafe impl Sync for MappedDataset {}

impl MappedDataset {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Zero-copy reference into the memory-mapped archive.
    #[inline]
    pub fn archived(&self) -> &ArchivedUrbanDataset {
        unsafe { &*self.archived }
    }

    pub fn building_count(&self) -> u64 {
        self.archived().header.building_count
    }

    /// Materialize an owned copy — only use when crossing process boundaries.
    pub fn to_owned(&self) -> UrbanDataset {
        rkyv::deserialize::<UrbanDataset, rkyv::rancor::Error>(self.archived())
            .expect("deserialization of validated archive must succeed")
    }
}

impl Drop for MappedDataset {
    fn drop(&mut self) {
        self.archived = std::ptr::null();
    }
}

fn access_archived(mmap: &Mmap, path: &Path) -> Result<*const ArchivedUrbanDataset, StorageError> {
    let bytes = mmap.as_ref();

    #[cfg(feature = "validation")]
    {
        let archived = rkyv::access::<ArchivedUrbanDataset, rkyv::rancor::Error>(bytes)
            .map_err(|e| StorageError::ArchiveAccess {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        if !archived.header.is_valid() {
            return Err(StorageError::InvalidHeader {
                path: path.to_path_buf(),
                reason: "magic or version mismatch".into(),
            });
        }

        return Ok(archived as *const ArchivedUrbanDataset);
    }

    #[cfg(not(feature = "validation"))]
    {
        let archived = rkyv::access_unchecked::<ArchivedUrbanDataset>(bytes);
        if !archived.header.is_valid() {
            return Err(StorageError::InvalidHeader {
                path: path.to_path_buf(),
                reason: "magic or version mismatch".into(),
            });
        }
        Ok(archived as *const ArchivedUrbanDataset)
    }
}

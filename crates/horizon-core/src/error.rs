use thiserror::Error;

use horizon_storage::StorageError;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error("spatial index build failed: {0}")]
    IndexBuild(String),

    #[error("query out of dataset bounds")]
    OutOfBounds,

    #[error("no dataset loaded")]
    NoDataset,
}

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

    #[error("invalid coastline geometry: {0}")]
    InvalidCoastline(String),

    #[error("horizon blockage analysis failed: {0}")]
    HorizonAnalysis(String),
}

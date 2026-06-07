use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("failed to open dataset at {path}: {source}")]
    Open {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to memory-map dataset at {path}: {source}")]
    Map {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("dataset at {path} is empty")]
    Empty { path: PathBuf },

    #[error("invalid dataset header at {path}: {reason}")]
    InvalidHeader { path: PathBuf, reason: String },

    #[error("archive access failed at {path}: {reason}")]
    ArchiveAccess { path: PathBuf, reason: String },

    #[error("archive validation failed at {path}: {source}")]
    Validation {
        path: PathBuf,
        source: bytecheck::Error,
    },
}

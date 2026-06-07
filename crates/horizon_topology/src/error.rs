use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopologyError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error("failed to decode footprint geometry: {0}")]
    WkbDecode(String),

    #[error("footprint ring is empty")]
    EmptyRing,
}

//! PostGIS-backed urban topology persistence layer.
//!
//! Provides compile-time verified sqlx queries against a PostGIS instance.
//! This crate is decoupled from the gRPC transport layer.

mod error;
mod ewkb;
mod model;
mod neighborhood;
mod repo;

pub use error::TopologyError;
pub use model::{BoundingBox3D, UrbanTopologyFeature};
pub use neighborhood::{CoastalNeighborhood, DARYAKENAR_ENVELOPE, IRANSHAHR_ENVELOPE};
pub use repo::UrbanTopologyRepo;

use sqlx::postgres::PgPoolOptions;

/// Establish a PostgreSQL connection pool from `DATABASE_URL`.
#[tracing::instrument]
pub async fn connect(database_url: &str) -> Result<sqlx::PgPool, TopologyError> {
    PgPoolOptions::new()
        .max_connections(16)
        .connect(database_url)
        .await
        .map_err(TopologyError::from)
}

/// Run pending sqlx migrations against the connected pool.
#[tracing::instrument(skip(pool))]
pub async fn migrate(pool: &sqlx::PgPool) -> Result<(), TopologyError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(TopologyError::from)
}

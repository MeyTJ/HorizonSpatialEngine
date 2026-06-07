mod adapter;

use std::sync::Arc;

use anyhow::Context;
use horizon_api::{SpatialComputeService, SpatialService};
use horizon_topology::{connect, migrate, UrbanTopologyRepo};
use horizon_transport::{serve, TransportConfig, TransportServices};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("horizon=info".parse()?))
        .init();

    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        let pool = connect(&database_url)
            .await
            .context("failed to connect to PostGIS")?;
        migrate(&pool)
            .await
            .context("failed to run database migrations")?;

        let topology_repo = UrbanTopologyRepo::new(pool);
        match topology_repo.fetch_coastal_neighborhoods().await {
            Ok(features) => {
                info!(
                    feature_count = features.len(),
                    "PostGIS coastal topology preloaded"
                );
            }
            Err(err) => {
                warn!(error = %err, "PostGIS preload skipped — table may be empty");
            }
        }
    } else {
        warn!("DATABASE_URL not set — PostGIS integration disabled");
    }

    let listen_addr = std::env::var("HORIZON_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".into())
        .parse()
        .context("invalid HORIZON_LISTEN_ADDR")?;

    let adapter = Arc::new(adapter::CoreAdapter::new());

    info!("HorizonSpatialEngine starting");
    serve(
        TransportConfig { listen_addr },
        TransportServices {
            spatial: adapter.clone(),
            compute: adapter,
        },
    )
    .await?;

    Ok(())
}

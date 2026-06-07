use sqlx::PgPool;
use tracing::{info, instrument};

use crate::model::{BoundingBox3D, UrbanTopologyFeature};
use crate::neighborhood::{CoastalNeighborhood, DARYAKENAR_ENVELOPE, IRANSHAHR_ENVELOPE};
use crate::ewkb::{decode_ewkb_footprint, to_horizon_polygon};
use crate::TopologyError;

/// PostGIS repository for coastal urban topology features.
pub struct UrbanTopologyRepo {
    pool: PgPool,
}

impl UrbanTopologyRepo {
    #[instrument(skip(pool))]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Fetch 3D bounding boxes and polygon footprints for Daryakenar and Iranshahr.
    ///
    /// Geometries are spatially pre-filtered in PostGIS via `ST_Intersects` +
    /// `ST_MakeEnvelope` before any geometry bytes cross the wire into Rust memory.
    #[instrument(skip(self))]
    pub async fn fetch_coastal_neighborhoods(
        &self,
    ) -> Result<Vec<UrbanTopologyFeature>, TopologyError> {
        let daryakenar = DARYAKENAR_ENVELOPE;
        let iranshahr = IRANSHAHR_ENVELOPE;

        let rows = sqlx::query_as!(
            UrbanTopologyRow,
            r#"
            SELECT
                t.id,
                t.neighborhood_name,
                ST_XMin(ST_Envelope(t.footprint)) AS "bbox_min_x!",
                ST_YMin(ST_Envelope(t.footprint)) AS "bbox_min_y!",
                t.elevation_min                     AS "bbox_min_z!",
                ST_XMax(ST_Envelope(t.footprint)) AS "bbox_max_x!",
                ST_YMax(ST_Envelope(t.footprint)) AS "bbox_max_y!",
                t.elevation_max                     AS "bbox_max_z!",
                ST_AsEWKB(t.footprint)              AS "footprint_wkb!"
            FROM urban_topology AS t
            WHERE
                (
                    t.neighborhood_name = $9
                    AND ST_Intersects(
                        t.footprint,
                        ST_MakeEnvelope($1, $2, $3, $4, $10)
                    )
                )
                OR
                (
                    t.neighborhood_name = $11
                    AND ST_Intersects(
                        t.footprint,
                        ST_MakeEnvelope($5, $6, $7, $8, $10)
                    )
                )
            ORDER BY t.neighborhood_name, t.id
            "#,
            daryakenar.min_lon,
            daryakenar.min_lat,
            daryakenar.max_lon,
            daryakenar.max_lat,
            iranshahr.min_lon,
            iranshahr.min_lat,
            iranshahr.max_lon,
            iranshahr.max_lat,
            CoastalNeighborhood::Daryakenar.name(),
            daryakenar.srid,
            CoastalNeighborhood::Iranshahr.name(),
        )
        .fetch_all(&self.pool)
        .await?;

        info!(row_count = rows.len(), "coastal topology rows fetched from PostGIS");

        rows.into_iter().map(row_to_feature).collect()
    }

    /// Fetch topology for a single coastal neighborhood envelope.
    #[instrument(skip(self), fields(neighborhood = neighborhood.name()))]
    pub async fn fetch_neighborhood(
        &self,
        neighborhood: CoastalNeighborhood,
    ) -> Result<Vec<UrbanTopologyFeature>, TopologyError> {
        let envelope = neighborhood.envelope();

        let rows = sqlx::query_as!(
            UrbanTopologyRow,
            r#"
            SELECT
                t.id,
                t.neighborhood_name,
                ST_XMin(ST_Envelope(t.footprint)) AS "bbox_min_x!",
                ST_YMin(ST_Envelope(t.footprint)) AS "bbox_min_y!",
                t.elevation_min                     AS "bbox_min_z!",
                ST_XMax(ST_Envelope(t.footprint)) AS "bbox_max_x!",
                ST_YMax(ST_Envelope(t.footprint)) AS "bbox_max_y!",
                t.elevation_max                     AS "bbox_max_z!",
                ST_AsEWKB(t.footprint)              AS "footprint_wkb!"
            FROM urban_topology AS t
            WHERE
                t.neighborhood_name = $1
                AND ST_Intersects(
                    t.footprint,
                    ST_MakeEnvelope($2, $3, $4, $5, $6)
                )
            ORDER BY t.id
            "#,
            neighborhood.name(),
            envelope.min_lon,
            envelope.min_lat,
            envelope.max_lon,
            envelope.max_lat,
            envelope.srid,
        )
        .fetch_all(&self.pool)
        .await?;

        info!(
            row_count = rows.len(),
            neighborhood = neighborhood.name(),
            "neighborhood topology rows fetched"
        );

        rows.into_iter().map(row_to_feature).collect()
    }
}

struct UrbanTopologyRow {
    id: i64,
    neighborhood_name: String,
    bbox_min_x: f64,
    bbox_min_y: f64,
    bbox_min_z: f64,
    bbox_max_x: f64,
    bbox_max_y: f64,
    bbox_max_z: f64,
    footprint_wkb: Vec<u8>,
}

fn row_to_feature(row: UrbanTopologyRow) -> Result<UrbanTopologyFeature, TopologyError> {
    let ring = decode_ewkb_footprint(&row.footprint_wkb)?;
    let footprint = to_horizon_polygon(&ring, row.bbox_min_z, row.bbox_max_z);

    Ok(UrbanTopologyFeature {
        id: row.id,
        neighborhood_name: row.neighborhood_name,
        bbox: BoundingBox3D {
            min_x: row.bbox_min_x,
            min_y: row.bbox_min_y,
            min_z: row.bbox_min_z,
            max_x: row.bbox_max_x,
            max_y: row.bbox_max_y,
            max_z: row.bbox_max_z,
        },
        footprint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coastal_envelopes_are_valid_wgs84_bounds() {
        for envelope in [DARYAKENAR_ENVELOPE, IRANSHAHR_ENVELOPE] {
            assert!(envelope.min_lon < envelope.max_lon);
            assert!(envelope.min_lat < envelope.max_lat);
            assert_eq!(envelope.srid, 4326);
        }
    }
}

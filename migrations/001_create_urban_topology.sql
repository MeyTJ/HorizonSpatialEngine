CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS urban_topology (
    id              BIGSERIAL PRIMARY KEY,
    neighborhood_name VARCHAR(128) NOT NULL,
    feature_name    VARCHAR(256),
    footprint       GEOMETRY(Polygon, 4326) NOT NULL,
    elevation_min   DOUBLE PRECISION NOT NULL DEFAULT 0,
    elevation_max   DOUBLE PRECISION NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_elevation_range CHECK (elevation_max >= elevation_min)
);

CREATE INDEX IF NOT EXISTS idx_urban_topology_footprint_gist
    ON urban_topology USING GIST (footprint);

CREATE INDEX IF NOT EXISTS idx_urban_topology_neighborhood
    ON urban_topology (neighborhood_name);

COMMENT ON TABLE urban_topology IS
    'Urban building footprints for coastal neighborhood spatial compute pipelines.';

COMMENT ON COLUMN urban_topology.footprint IS
    'WGS84 polygon footprint; Z extent stored in elevation_min/elevation_max.';

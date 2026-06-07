/// WGS84 search envelope for spatial pre-filtering via `ST_MakeEnvelope`.
#[derive(Debug, Clone, Copy)]
pub struct SearchEnvelope {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
    pub srid: i32,
}

impl SearchEnvelope {
    pub const fn new(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64, srid: i32) -> Self {
        Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
            srid,
        }
    }
}

/// Supported coastal neighborhoods for topology ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoastalNeighborhood {
    Daryakenar,
    Iranshahr,
}

impl CoastalNeighborhood {
    pub fn name(self) -> &'static str {
        match self {
            Self::Daryakenar => "Daryakenar",
            Self::Iranshahr => "Iranshahr",
        }
    }

    pub fn envelope(self) -> SearchEnvelope {
        match self {
            Self::Daryakenar => DARYAKENAR_ENVELOPE,
            Self::Iranshahr => IRANSHAHR_ENVELOPE,
        }
    }
}

/// Daryakenar — Caspian Sea coast, Mazandaran province (~36.71°N, 52.66°E).
pub const DARYAKENAR_ENVELOPE: SearchEnvelope = SearchEnvelope::new(
    52.595, 36.675, 52.725, 36.745, 4326,
);

/// Iranshahr — Gulf of Oman coast, Sistan-Baluchestan province (~27.20°N, 60.68°E).
pub const IRANSHAHR_ENVELOPE: SearchEnvelope = SearchEnvelope::new(
    60.650, 27.180, 60.720, 27.240, 4326,
);

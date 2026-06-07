//! Visual justice analytical domain logic.

mod horizon_blockage;
mod raycast;
mod scratch;

pub use horizon_blockage::{
    calculate_horizon_blockage, HorizonBlockageResult, COASTAL_HIGHRISE_MIN_HEIGHT,
    HORIZON_RAY_COUNT,
};
pub use raycast::{direction_from_bearing, ray_segment_distance};

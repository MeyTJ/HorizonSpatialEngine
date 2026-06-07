use std::f64::consts::PI;

use horizon_geometry::{Coordinate, LineString};
use rayon::prelude::*;
use tracing::instrument;

use crate::error::CoreError;
use crate::query::QueryBounds;
use crate::spatial::SpatialIndex;
use crate::visual_justice_metrics::raycast::{
    direction_from_bearing, ray_polygon_distance, ray_polyline_distance, sight_line_elevation,
    EPSILON,
};
use crate::visual_justice_metrics::scratch::FootprintScratch;

/// Number of rays cast per horizon blockage analysis.
pub const HORIZON_RAY_COUNT: usize = 1_000;

/// Minimum building height (metres) to qualify as a coastal high-rise obstruction.
pub const COASTAL_HIGHRISE_MIN_HEIGHT: f64 = 12.0;

/// Width of the spatial query corridor around each ray (metres).
const RAY_CORRIDOR_HALF_WIDTH: f64 = 2.0;

/// Result of a visual horizon blockage analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizonBlockageResult {
    pub obstruction_percentage: f64,
    pub rays_cast: usize,
    pub rays_obstructed: usize,
}

/// Cast [`HORIZON_RAY_COUNT`] parallel rays from a citizen viewpoint toward a target
/// coastline and return the exact percentage of the visual horizon obstructed by
/// coastal high-rise polygons indexed in `SpatialIndex`.
#[instrument(skip(index, target_coastline), fields(
    viewpoint_x = viewpoint.x,
    viewpoint_y = viewpoint.y,
    viewpoint_z = viewpoint.z,
    coastline_vertices = target_coastline.len()
))]
pub fn calculate_horizon_blockage(
    index: &SpatialIndex,
    viewpoint: Coordinate,
    target_coastline: &LineString,
) -> Result<HorizonBlockageResult, CoreError> {
    if target_coastline.len() < 2 {
        return Err(CoreError::InvalidCoastline(
            "coastline must contain at least two points".into(),
        ));
    }

    let (min_bearing, max_bearing) = coastline_bearing_arc(viewpoint, target_coastline)?;
    let bearings = linspace_bearings(min_bearing, max_bearing, HORIZON_RAY_COUNT);

    let coast_xs: Vec<f64> = target_coastline.points.iter().map(|p| p.x).collect();
    let coast_ys: Vec<f64> = target_coastline.points.iter().map(|p| p.y).collect();

    let rays_obstructed = bearings
        .par_iter()
        .filter(|&&bearing| {
            ray_is_obstructed(index, viewpoint, bearing, &coast_xs, &coast_ys, target_coastline)
        })
        .count();

    let obstruction_percentage = (rays_obstructed as f64 / HORIZON_RAY_COUNT as f64) * 100.0;

    Ok(HorizonBlockageResult {
        obstruction_percentage,
        rays_cast: HORIZON_RAY_COUNT,
        rays_obstructed,
    })
}

fn ray_is_obstructed(
    index: &SpatialIndex,
    viewpoint: Coordinate,
    bearing: f64,
    coast_xs: &[f64],
    coast_ys: &[f64],
    coastline: &LineString,
) -> bool {
    let (dir_x, dir_y) = direction_from_bearing(bearing);

    let Some(coast_distance) =
        ray_polyline_distance(viewpoint.x, viewpoint.y, dir_x, dir_y, coast_xs, coast_ys)
    else {
        return false;
    };

    if coast_distance <= EPSILON {
        return false;
    }

    let coast_z = interpolate_coast_elevation(viewpoint, dir_x, dir_y, coast_distance, coastline);
    let corridor = ray_corridor_bounds(viewpoint.x, viewpoint.y, dir_x, dir_y, coast_distance);
    let candidates = index.query_intersects(&corridor);
    let mut scratch = FootprintScratch::default();

    for feature_idx in candidates {
        let Some(building) = index.building(feature_idx) else {
            continue;
        };

        if building.height < COASTAL_HIGHRISE_MIN_HEIGHT {
            continue;
        }

        let verts = building.footprint.vertices.as_slice();
        if !scratch.load_vertices(verts) {
            continue;
        }

        let Some(hit_distance) = ray_polygon_distance(
            viewpoint.x,
            viewpoint.y,
            dir_x,
            dir_y,
            scratch.xs(),
            scratch.ys(),
        ) else {
            continue;
        };

        if hit_distance >= coast_distance {
            continue;
        }

        let sight_z = sight_line_elevation(viewpoint.z, coast_z, hit_distance, coast_distance);
        let building_top = building
            .footprint
            .elevation_max
            .max(building.height);

        if building_top > sight_z {
            return true;
        }
    }

    false
}

fn coastline_bearing_arc(
    viewpoint: Coordinate,
    coastline: &LineString,
) -> Result<(f64, f64), CoreError> {
    let mut bearings: Vec<f64> = coastline
        .points
        .iter()
        .map(|point| bearing_to(viewpoint, *point))
        .collect();

    if bearings.is_empty() {
        return Err(CoreError::InvalidCoastline(
            "coastline has no bearings".into(),
        ));
    }

    bearings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = bearings[0];
    let max = bearings[bearings.len() - 1];

    Ok((min, max))
}

#[inline]
fn bearing_to(from: Coordinate, to: Coordinate) -> f64 {
    (to.y - from.y).atan2(to.x - from.x)
}

fn linspace_bearings(min: f64, max: f64, count: usize) -> Vec<f64> {
    if count <= 1 {
        return vec![min];
    }
    let step = (max - min) / (count - 1) as f64;
    (0..count).map(|i| min + i as f64 * step).collect()
}

fn interpolate_coast_elevation(
    viewpoint: Coordinate,
    dir_x: f64,
    dir_y: f64,
    coast_distance: f64,
    coastline: &LineString,
) -> f64 {
    let target_x = viewpoint.x + dir_x * coast_distance;
    let target_y = viewpoint.y + dir_y * coast_distance;

    let mut best_dist = f64::INFINITY;
    let mut coast_z = 0.0_f64;

    for point in &coastline.points {
        let dx = point.x - target_x;
        let dy = point.y - target_y;
        let dist = dx * dx + dy * dy;
        if dist < best_dist {
            best_dist = dist;
            coast_z = point.z;
        }
    }

    coast_z
}

fn ray_corridor_bounds(
    origin_x: f64,
    origin_y: f64,
    dir_x: f64,
    dir_y: f64,
    max_distance: f64,
) -> QueryBounds {
    let end_x = origin_x + dir_x * max_distance;
    let end_y = origin_y + dir_y * max_distance;
    let pad = RAY_CORRIDOR_HALF_WIDTH;

    let perp_x = -dir_y * pad;
    let perp_y = dir_x * pad;

    let min_x = origin_x
        .min(end_x)
        .min(origin_x + perp_x)
        .min(end_x + perp_x)
        .min(origin_x - perp_x)
        .min(end_x - perp_x)
        - pad;
    let max_x = origin_x
        .max(end_x)
        .max(origin_x + perp_x)
        .max(end_x + perp_x)
        .max(origin_x - perp_x)
        .max(end_x - perp_x)
        + pad;
    let min_y = origin_y
        .min(end_y)
        .min(origin_y + perp_y)
        .min(end_y + perp_y)
        .min(origin_y - perp_y)
        .min(end_y - perp_y)
        - pad;
    let max_y = origin_y
        .max(end_y)
        .max(origin_y + perp_y)
        .max(end_y + perp_y)
        .max(origin_y - perp_y)
        .max(end_y - perp_y)
        + pad;

    QueryBounds::new(min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearing_arc_spans_coastline() {
        let viewpoint = Coordinate::new(0.0, 0.0, 1.7);
        let coastline = LineString::new(vec![
            Coordinate::new(10.0, -5.0, 0.0),
            Coordinate::new(10.0, 5.0, 0.0),
        ]);
        let (min, max) = coastline_bearing_arc(viewpoint, &coastline).expect("arc");
        assert!(min < 0.0);
        assert!(max > 0.0);
    }

    #[test]
    fn linspace_produces_ray_count() {
        let bearings = linspace_bearings(-PI / 4.0, PI / 4.0, HORIZON_RAY_COUNT);
        assert_eq!(bearings.len(), HORIZON_RAY_COUNT);
    }
}

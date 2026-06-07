//! SIMD-accelerated ray/segment intersection primitives.

use wide::f64x4;

pub const EPSILON: f64 = 1e-12;

/// Unit direction vector from a bearing angle (radians, 0 = +X, CCW).
#[inline]
pub fn direction_from_bearing(bearing: f64) -> (f64, f64) {
    (bearing.cos(), bearing.sin())
}

/// 2D cross product (scalar z-component).
#[inline]
pub fn cross(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ax * by - ay * bx
}

/// Distance along a unit ray to the nearest intersection with a line segment.
pub fn ray_segment_distance(
    origin_x: f64,
    origin_y: f64,
    dir_x: f64,
    dir_y: f64,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
) -> Option<f64> {
    let seg_x = bx - ax;
    let seg_y = by - ay;
    let origin_ax = origin_x - ax;
    let origin_ay = origin_y - ay;

    let denom = cross(dir_x, dir_y, seg_x, seg_y);
    if denom.abs() < EPSILON {
        return None;
    }

    let t = cross(origin_ax, origin_ay, seg_x, seg_y) / denom;
    let u = cross(origin_ax, origin_ay, dir_x, dir_y) / denom;

    if t >= 0.0 && u >= 0.0 && u <= 1.0 {
        Some(t)
    } else {
        None
    }
}

/// Batch-test a unit ray against up to four segments using portable SIMD (`wide`).
pub fn ray_segments_distance_simd(
    origin_x: f64,
    origin_y: f64,
    dir_x: f64,
    dir_y: f64,
    ax: [f64; 4],
    ay: [f64; 4],
    bx: [f64; 4],
    by: [f64; 4],
    count: usize,
) -> Option<f64> {
    let o_x = f64x4::splat(origin_x);
    let o_y = f64x4::splat(origin_y);
    let d_x = f64x4::splat(dir_x);
    let d_y = f64x4::splat(dir_y);

    let a_x = f64x4::new(ax);
    let a_y = f64x4::new(ay);
    let b_x = f64x4::new(bx);
    let b_y = f64x4::new(by);

    let seg_x = b_x - a_x;
    let seg_y = b_y - a_y;
    let origin_ax = o_x - a_x;
    let origin_ay = o_y - a_y;

    let denom = d_x * seg_y - d_y * seg_x;
    let t = (origin_ax * seg_y - origin_ay * seg_x) / denom;
    let u = (origin_ax * d_y - origin_ay * d_x) / denom;

    let t_arr = t.to_array();
    let u_arr = u.to_array();
    let denom_arr = denom.to_array();

    let mut best = f64::INFINITY;
    for i in 0..count {
        if denom_arr[i].abs() > EPSILON
            && t_arr[i] >= 0.0
            && u_arr[i] >= 0.0
            && u_arr[i] <= 1.0
            && t_arr[i] < best
        {
            best = t_arr[i];
        }
    }

    if best.is_finite() {
        Some(best)
    } else {
        None
    }
}

/// Minimum distance along a unit ray to a closed polygon ring (zero-copy vertex slice).
pub fn ray_polygon_distance(
    origin_x: f64,
    origin_y: f64,
    dir_x: f64,
    dir_y: f64,
    xs: &[f64],
    ys: &[f64],
) -> Option<f64> {
    if xs.len() < 3 || xs.len() != ys.len() {
        return None;
    }

    let mut best = f64::INFINITY;
    let n = xs.len();
    let mut i = 0;

    while i + 4 <= n {
        let ax = [xs[i], xs[(i + 1) % n], xs[(i + 2) % n], xs[(i + 3) % n]];
        let ay = [ys[i], ys[(i + 1) % n], ys[(i + 2) % n], ys[(i + 3) % n]];
        let bx = [
            xs[(i + 1) % n],
            xs[(i + 2) % n],
            xs[(i + 3) % n],
            xs[(i + 4) % n],
        ];
        let by = [
            ys[(i + 1) % n],
            ys[(i + 2) % n],
            ys[(i + 3) % n],
            ys[(i + 4) % n],
        ];

        if let Some(dist) =
            ray_segments_distance_simd(origin_x, origin_y, dir_x, dir_y, ax, ay, bx, by, 4)
        {
            best = best.min(dist);
        }
        i += 4;
    }

    while i < n {
        let j = (i + 1) % n;
        if let Some(dist) =
            ray_segment_distance(origin_x, origin_y, dir_x, dir_y, xs[i], ys[i], xs[j], ys[j])
        {
            best = best.min(dist);
        }
        i += 1;
    }

    if best.is_finite() {
        Some(best)
    } else {
        None
    }
}

/// Minimum distance along a unit ray to a polyline ring (closed or open).
pub fn ray_polyline_distance(
    origin_x: f64,
    origin_y: f64,
    dir_x: f64,
    dir_y: f64,
    xs: &[f64],
    ys: &[f64],
) -> Option<f64> {
    if xs.len() < 2 || xs.len() != ys.len() {
        return None;
    }

    let mut best = f64::INFINITY;
    let segment_count = xs.len() - 1;
    let mut i = 0;

    while i + 4 <= segment_count {
        let ax = [xs[i], xs[i + 1], xs[i + 2], xs[i + 3]];
        let ay = [ys[i], ys[i + 1], ys[i + 2], ys[i + 3]];
        let bx = [xs[i + 1], xs[i + 2], xs[i + 3], xs[i + 4]];
        let by = [ys[i + 1], ys[i + 2], ys[i + 3], ys[i + 4]];

        if let Some(dist) =
            ray_segments_distance_simd(origin_x, origin_y, dir_x, dir_y, ax, ay, bx, by, 4)
        {
            best = best.min(dist);
        }
        i += 4;
    }

    while i < segment_count {
        if let Some(dist) = ray_segment_distance(
            origin_x,
            origin_y,
            dir_x,
            dir_y,
            xs[i],
            ys[i],
            xs[i + 1],
            ys[i + 1],
        ) {
            best = best.min(dist);
        }
        i += 1;
    }

    if best.is_finite() {
        Some(best)
    } else {
        None
    }
}

/// Sight-line elevation at distance `d` along a ray toward the coast.
#[inline]
pub fn sight_line_elevation(eye_z: f64, coast_z: f64, d: f64, coast_distance: f64) -> f64 {
    if coast_distance <= EPSILON {
        return eye_z;
    }
    eye_z + (d / coast_distance) * (coast_z - eye_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_segment() {
        let dist = ray_segment_distance(0.0, 0.0, 1.0, 0.0, 5.0, -1.0, 5.0, 1.0);
        assert_eq!(dist, Some(5.0));
    }

    #[test]
    fn simd_matches_scalar() {
        let dist_scalar = ray_segment_distance(0.0, 0.0, 1.0, 0.0, 5.0, -1.0, 5.0, 1.0);
        let dist_simd = ray_segments_distance_simd(
            0.0,
            0.0,
            1.0,
            0.0,
            [5.0, 0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0, 0.0],
            [5.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            1,
        );
        assert_eq!(dist_scalar, dist_simd);
    }
}

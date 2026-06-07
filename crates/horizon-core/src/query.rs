/// Axis-aligned query region in map coordinates.
#[derive(Debug, Clone, Copy)]
pub struct QueryBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl QueryBounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
            min_z: f64::NEG_INFINITY,
            max_z: f64::INFINITY,
        }
    }

    pub fn with_elevation(mut self, min_z: f64, max_z: f64) -> Self {
        self.min_z = min_z;
        self.max_z = max_z;
        self
    }

    pub fn area(&self) -> f64 {
        (self.max_x - self.min_x).abs() * (self.max_y - self.min_y).abs()
    }
}

/// Result of a geometric intersection query.
#[derive(Debug, Clone)]
pub struct IntersectionResult {
    pub building_ids: Vec<u64>,
    pub matched_count: usize,
    pub query_area: f64,
}

/// Visual accessibility metric for an observer point.
#[derive(Debug, Clone)]
pub struct AccessibilityResult {
    pub visible_building_count: usize,
    pub skyline_obstruction_ratio: f64,
    pub mean_view_distance: f64,
}

/// Spatial query dispatched to the compute engine.
#[derive(Debug, Clone)]
pub enum SpatialQuery {
    Intersect(QueryBounds),
    Accessibility {
        observer_x: f64,
        observer_y: f64,
        observer_z: f64,
        radius: f64,
    },
}

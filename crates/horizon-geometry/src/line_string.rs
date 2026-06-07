use crate::Coordinate;

/// An ordered sequence of coordinates representing a polyline (e.g. a coastline).
#[derive(Debug, Clone, PartialEq)]
pub struct LineString {
    pub points: Vec<Coordinate>,
}

impl LineString {
    pub fn new(points: Vec<Coordinate>) -> Self {
        Self { points }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn segments(&self) -> impl Iterator<Item = (Coordinate, Coordinate)> + '_ {
        self.points.windows(2).map(|w| (w[0], w[1]))
    }
}

use num_traits::Float;

use crate::{IsClose, Tolerance, Vertex};

/// A point in the plain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<T> {
    /// The horizontal coordinate of the point.
    pub x: T,
    /// The vertical coordinate of the point.
    pub y: T,
}

impl<T> From<[T; 2]> for Point<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Self { x, y }
    }
}

impl<T> Vertex for Point<T>
where
    T: Copy + Float,
{
    type Scalar = T;

    fn distance(&self, rhs: &Self) -> Self::Scalar {
        ((self.x - rhs.x).powi(2) + (self.y - rhs.y).powi(2)).sqrt()
    }
}

impl<T> IsClose for Point<T>
where
    T: IsClose<Tolerance = Tolerance<T>>,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, rhs: &Self, tolerance: &Self::Tolerance) -> bool {
        self.x.is_close(&rhs.x, tolerance) && self.y.is_close(&rhs.y, tolerance)
    }
}

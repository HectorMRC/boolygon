use num_traits::Float;

use crate::{Element, IsClose, Tolerance};

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

impl<T> Element for Point<T>
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
    T: IsClose<Scalar = T> + Copy,
{
    type Scalar = T;

    fn is_close(&self, rhs: &Self, tolerance: &Tolerance<Self::Scalar>) -> bool {
        self.x.is_close(&rhs.x, tolerance) && self.y.is_close(&rhs.y, tolerance)
    }
}

/// A constructor macro for the cartesian [`Point`].
#[macro_export]
macro_rules! cartesian_point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y }
    };
}

pub use cartesian_point;

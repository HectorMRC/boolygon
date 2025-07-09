use num_traits::Float;

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

impl<T> Point<T>
where
    T: Copy + Float,
{
    /// Returns the distance between self and rhs.
    pub(crate) fn distance(&self, rhs: &Self) -> T {
        ((self.x - rhs.x).powi(2) + (self.y - rhs.y).powi(2)).sqrt()
    }
}

#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y }
    };
}

pub use point;

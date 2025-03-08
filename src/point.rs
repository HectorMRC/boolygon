/// Represents a point in a plain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point<T = f64> {
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

#[macro_export]
macro_rules! point {
    ($x:expr, $y:expr) => {
        Point { x: $x, y: $y }
    };
}

pub use point;

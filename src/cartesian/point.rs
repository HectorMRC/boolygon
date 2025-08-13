use std::ops::{Add, Mul, Sub};

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

impl<T> Add for Point<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Point<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Point<T>
where
    T: Copy + Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Point {
            x: rhs * self.x,
            y: rhs * self.y,
        }
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

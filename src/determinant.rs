use std::ops::{Mul, Sub};

use num_traits::{Float, Signed};

use crate::{point::Point, polygon::Segment};

/// The scalar value representing the determinant between two vectors.
#[derive(Debug, Copy, Clone)]
pub struct Determinant<T>(T);

impl<T> From<[&Point<T>; 3]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Returns the determinant of the direction vectors `AB` and `AC`.
    fn from([a, b, c]: [&Point<T>; 3]) -> Self {
        Self((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y))
    }
}

impl<T> From<[&Segment<'_, T>; 2]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Returns the determinant of `AB`.
    fn from([a, b]: [&Segment<'_, T>; 2]) -> Self {
        Self((a.to.x - a.from.x) * (b.to.y - b.from.y) - (b.to.x - b.from.x) * (a.to.y - a.from.y))
    }
}

impl<T> Determinant<T>
where
    T: Float,
{
    /// Returns `true` if `self` is equal to the additive identity.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T> Determinant<T>
where
    T: Signed,
{
    /// Returns the sign of the number.
    pub fn signum(&self) -> T {
        self.0.signum()
    }

    /// Returns true if the number is positive and false if the number is zero or negative.
    pub fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    /// Returns true if the number is negative and false if the number is zero or positive.
    pub fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

impl<T> Determinant<T> {
    /// Returns the inner value of self.
    pub fn into_inner(self) -> T {
        self.0
    }
}

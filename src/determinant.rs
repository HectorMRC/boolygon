use std::ops::{Mul, Sub};

use crate::{point::Point, polygon::Segment};

/// The scalar value representing the determinant between two vectors.
#[derive(Debug, Copy, Clone)]
pub struct Determinant<T>(T);

impl<T> From<[&Point<T>; 3]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Being `A`, `B` and `C` the given [`Point`]s, returns the determinant of the direction
    /// vectors `AB` and `AC`.
    fn from([a, b, c]: [&Point<T>; 3]) -> Self {
        Self((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y))
    }
}

impl<T> From<(&Segment<'_, T>, &Point<T>)> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Being `A` and `B` the endpoints of the given [`Segment`], and `C` the given [`Point`],
    /// returns the determinant of the direction vector `AB` and `AC`.
    fn from((segment, point): (&Segment<'_, T>, &Point<T>)) -> Self {
        Self::from([&segment.from, &segment.to, point])
    }
}

impl<T> From<[&Segment<'_, T>; 2]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Returns the determinant of the direction vectors of the given [`Segment`]s.
    fn from([a, b]: [&Segment<'_, T>; 2]) -> Self {
        Self((a.to.x - a.from.x) * (b.to.y - b.from.y) - (b.to.x - b.from.x) * (a.to.y - a.from.y))
    }
}

impl<T> Determinant<T> {
    /// Returns the inner value of self.
    pub fn into_inner(self) -> T {
        self.0
    }
}

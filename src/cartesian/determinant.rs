use std::ops::{Mul, Sub};

use super::{Point, Segment};

/// The scalar value representing the determinant of a matrix.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct Determinant<T>(T);

impl<T> From<[&Point<T>; 3]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Being `A`, `B` and `C` the given [`Point`]s, returns the determinant of the matrix
    /// representing the direction vectors `AB` and `AC`.
    fn from([a, b, c]: [&Point<T>; 3]) -> Self {
        Self((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y))
    }
}

impl<T> From<(&Segment<'_, T>, &Point<T>)> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Being `A` and `B` the endpoints of the given [`Segment`], and `C` the given [`Point`],
    /// returns the determinant of the matrix representing the direction vector `AB` and `AC`.
    fn from((segment, point): (&Segment<'_, T>, &Point<T>)) -> Self {
        Self::from([segment.from, segment.to, point])
    }
}

impl<T> From<[&Segment<'_, T>; 2]> for Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Returns the determinant of the matrix representing the direction vectors of the given
    /// [`Segment`]s.
    fn from([a, b]: [&Segment<'_, T>; 2]) -> Self {
        Self((a.to.x - a.from.x) * (b.to.y - b.from.y) - (b.to.x - b.from.x) * (a.to.y - a.from.y))
    }
}

impl<T> Determinant<T> {
    /// Returns the inner value of self.
    pub(super) fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::cartesian::{
        determinant::Determinant,
        point::{Point, cartesian_point},
    };

    #[test]
    fn determinant_of_vectors() {
        struct Test {
            name: &'static str,
            points: [Point<f64>; 3],
            want: Determinant<f64>,
        }

        vec![
            Test {
                name: "clockwise vectors",
                points: [
                    cartesian_point!(0., 1.),
                    cartesian_point!(0., 0.),
                    cartesian_point!(-1., 0.),
                ],
                want: Determinant(-1.),
            },
            Test {
                name: "counter-clockwise vectors",
                points: [
                    cartesian_point!(0., 1.),
                    cartesian_point!(0., 0.),
                    cartesian_point!(1., 0.),
                ],
                want: Determinant(1.),
            },
            Test {
                name: "colinear vectors",
                points: [
                    cartesian_point!(4., 4.),
                    cartesian_point!(2., 2.),
                    cartesian_point!(0., 0.),
                ],
                want: Determinant(0.),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let [a, b, c] = test.points;
            let got = Determinant::from([&a, &b, &c]);

            assert_eq!(
                got, test.want,
                "{}: got determinant = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }
}

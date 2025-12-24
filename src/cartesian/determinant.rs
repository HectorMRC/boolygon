use std::ops::{Mul, Sub};

use super::Point;

/// The scalar value representing the determinant of a matrix.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct Determinant<T>(T);

impl<T> Determinant<T>
where
    T: Copy + Sub<Output = T> + Mul<Output = T>,
{
    /// Returns the determinant of the two vectors starting at the given origin to the 
    /// respective endpoints.
    pub(crate) fn new(origin: &Point<T>, first: &Point<T>, second: &Point<T>) -> Self {
        Self((first.x - origin.x) * (second.y - origin.y) - (second.x - origin.x) * (first.y - origin.y))
    }
}

impl<T> Determinant<T> {
    /// Returns the inner value of self.
    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::cartesian::{determinant::Determinant, point::Point};

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
                points: [[0., 1.].into(), [0., 0.].into(), [-1., 0.].into()],
                want: Determinant(-1.),
            },
            Test {
                name: "counter-clockwise vectors",
                points: [[0., 1.].into(), [0., 0.].into(), [1., 0.].into()],
                want: Determinant(1.),
            },
            Test {
                name: "colinear vectors",
                points: [[4., 4.].into(), [2., 2.].into(), [0., 0.].into()],
                want: Determinant(0.),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let [a, b, c] = test.points;
            let got = Determinant::new(&a, &b, &c);

            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

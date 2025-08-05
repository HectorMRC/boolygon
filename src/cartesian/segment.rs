use num_traits::{Float, Signed};

use crate::{
    cartesian::{determinant::Determinant, Point},
    Edge, IsClose, Tolerance, Vertex as _,
};

/// The straight line between two endpoints.
#[derive(Debug)]
pub struct Segment<'a, T> {
    /// The first point in the segment.
    pub from: &'a Point<T>,
    /// The last point in the segment.
    pub to: &'a Point<T>,
}

impl<'a, T> Edge<'a> for Segment<'a, T>
where
    T: Signed + Float,
{
    type Vertex = Point<T>;

    fn new(from: &'a Self::Vertex, to: &'a Self::Vertex) -> Self {
        Self { from, to }
    }

    fn midpoint(&self) -> Self::Vertex {
        let two = T::one() + T::one();
        Point {
            x: (self.from.x + self.to.x) / two,
            y: (self.from.y + self.to.y) / two,
        }
    }

    fn contains(&self, point: &Self::Vertex, tolerance: &Tolerance<T>) -> bool {
        (self.from.distance(point) + self.to.distance(point)).is_close(&self.length(), tolerance)
    }

    fn intersection(&self, rhs: &Self, tolerance: &Tolerance<T>) -> Option<Self::Vertex> {
        let determinant = Determinant::from([self, rhs]).into_inner();

        if determinant.is_zero() {
            // When the two (infinte) lines are parallel or coincident, the determinant is zero.
            return self.collinear_common_point(rhs, tolerance);
        }

        let t = (self.from.x - rhs.from.x) * (rhs.from.y - rhs.to.y)
            - (self.from.y - rhs.from.y) * (rhs.from.x - rhs.to.x);

        // Predict if the division `t / determinant` will be in the range `[0,1]`
        if t.abs() > determinant.abs() || !t.is_zero() && t.signum() != determinant.signum() {
            return None;
        }

        let t = t / determinant;

        let u = -((self.from.x - self.to.x) * (self.from.y - rhs.from.y)
            - (self.from.y - self.to.y) * (self.from.x - rhs.from.x));

        // Predict if the division `u / determinant` will be in the range `[0,1]`
        if u.abs() > determinant.abs() || !u.is_zero() && u.signum() != determinant.signum() {
            return None;
        }

        Some(Point {
            x: self.from.x + t * (self.to.x - self.from.x),
            y: self.from.y + t * (self.to.y - self.from.y),
        })
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float,
{
    /// Being zero the determinant of self and rhs, returns the single common [`Point`] between
    /// them, if any.
    fn collinear_common_point(
        &self,
        rhs: &Segment<'_, T>,
        tolerance: &Tolerance<T>,
    ) -> Option<Point<T>> {
        if !rhs.contains(self.to, tolerance)
            && (self.from.is_close(rhs.from, tolerance) && !self.contains(rhs.to, tolerance)
                || self.from.is_close(rhs.to, tolerance) && !self.contains(rhs.from, tolerance))
        {
            return Some(*self.from);
        }

        if !rhs.contains(self.from, tolerance)
            && (self.to.is_close(rhs.from, tolerance) && !self.contains(rhs.to, tolerance)
                || self.to.is_close(rhs.to, tolerance) && !self.contains(rhs.from, tolerance))
        {
            return Some(*self.to);
        }

        None
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float,
{
    /// Returns the distance between the two endpoints of the segment.
    fn length(&self) -> T {
        self.from.distance(self.to)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cartesian::{Point, Segment},
        Edge,
    };

    #[test]
    fn segment_intersection() {
        struct Test<'a> {
            name: &'a str,
            segment: Segment<'a, f64>,
            rhs: Segment<'a, f64>,
            want: Option<Point<f64>>,
        }

        vec![
            Test {
                name: "crossing segments",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 0.].into(),
                },
                want: Some([2., 2.].into()),
            },
            Test {
                name: "segments starting at the same point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[0., 0.].into(),
                    to: &[-4., 4.].into(),
                },
                want: Some([0., 0.].into()),
            },
            Test {
                name: "connected segments",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[0., 0.].into(),
                },
                rhs: Segment {
                    from: &[0., 0.].into(),
                    to: &[-4., 4.].into(),
                },
                want: Some([0., 0.].into()),
            },
            Test {
                name: "collinear segments with common point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[-4., -4.].into(),
                    to: &[0., 0.].into(),
                },
                want: Some([0., 0.].into()),
            },
            Test {
                name: "collinear segments with no common point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[-4., -4.].into(),
                    to: &[-2., -2.].into(),
                },
                want: None,
            },
            Test {
                name: "segments ending at the same point",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[0., 0.].into(),
                },
                rhs: Segment {
                    from: &[-4., 4.].into(),
                    to: &[0., 0.].into(),
                },
                want: Some([0., 0.].into()),
            },
            Test {
                name: "parallel segments",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 8.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident segments when rhs is shorter",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[0., 0.].into(),
                    to: &[2., 2.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident segments when rhs is larger",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[0., 0.].into(),
                    to: &[8., 8.].into(),
                },
                want: None,
            },
            Test {
                name: "segment inside rhs",
                segment: Segment {
                    from: &[1., 1.].into(),
                    to: &[3., 3.].into(),
                },
                rhs: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                want: None,
            },
            Test {
                name: "rhs inside segment",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[1., 1.].into(),
                    to: &[3., 3.].into(),
                },
                want: None,
            },
            Test {
                name: "non-crossing segments",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[8., 8.].into(),
                },
                rhs: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 0.].into(),
                },
                want: None,
            },
            Test {
                name: "perpendicular segments",
                segment: Segment {
                    from: &[4., 0.].into(),
                    to: &[4., 4.].into(),
                },
                rhs: Segment {
                    from: &[2., 2.].into(),
                    to: &[6., 2.].into(),
                },
                want: Some([4., 2.].into()),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.segment.intersection(&test.rhs, &Default::default());
            assert_eq!(
                got, test.want,
                "{}: got intersection point = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }
}

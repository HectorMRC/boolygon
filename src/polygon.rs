use std::{
    cmp::Ordering,
    ops::{Mul, Sub},
};

use num_traits::{Float, Signed, Zero};

use crate::{determinant::Determinant, point::Point};

/// Represents the straight line between two consecutive vertices of a [`Polygon`].
pub(crate) struct Segment<'a, T> {
    /// The first point in the segment.
    pub from: &'a Point<T>,
    /// The last point in the segment.
    pub to: &'a Point<T>,
}

impl<'a, T> From<(&'a Point<T>, &'a Point<T>)> for Segment<'a, T> {
    fn from((from, to): (&'a Point<T>, &'a Point<T>)) -> Self {
        Self { from, to }
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float + Zero,
{
    /// Returns the distance between the two endpoints of the segment.
    fn length(&self) -> T {
        self.from.distance(self.to)
    }

    /// Returns true if, and only if, the given [`Point`] exists within the segment.
    fn contains(&self, point: &Point<T>) -> bool {
        Determinant::from((self, point)).into_inner().is_zero()
            && self.from.distance(point) <= self.length()
            && self.to.distance(point) <= self.length()
    }
    /// Returns the [`Point`] of intersection between self and the given segment, if any.
    fn intersection(&self, rhs: &Self) -> Option<Point<T>> {
        let determinant = Determinant::from([self, rhs]).into_inner();

        if determinant.is_zero() {
            // When the two (infinte) lines are parallel or coincident, the determinant is zero.
            return self.collinear_common_point(rhs);
        }

        let t = (self.from.x - rhs.from.x) * (rhs.from.y - rhs.to.y)
            - (self.from.y - rhs.from.y) * (rhs.from.x - rhs.to.x);

        // Predict if the division `t / determinant` will be in the range `[0,1]`
        if t.abs() > determinant.abs() || !t.is_zero() && t.signum() != determinant.signum() {
            return None;
        }

        let t = t / determinant;
        let u = -(self.from.x - self.to.x) * (self.from.y - rhs.from.y)
            - (self.from.y - self.to.y) * (self.from.x - rhs.from.x);

        // Predict if the division `u / determinant` will be in the range `[0,1]`
        if u.abs() > determinant.abs() || !u.is_zero() && u.signum() != determinant.signum() {
            return None;
        }

        Some(Point {
            x: self.from.x + t * (self.to.x - self.from.x),
            y: self.from.y + t * (self.to.y - self.from.y),
        })
    }

    /// Being zero the determinant of self and rhs, returns the single common [`Point`] between
    /// them, if any.
    fn collinear_common_point(&self, rhs: &Segment<'_, T>) -> Option<Point<T>> {
        let contains = |point| {
            // Optimized version of `Self::contains`:
            // No need to compute the cross product, by sharing an endpoint its guaranteed both
            // parallel segments are collinear.
            self.from.distance(point) <= self.length() && self.to.distance(point) <= self.length()
        };

        if (self.from == rhs.from && !contains(rhs.to))
            || (self.from == rhs.to && !contains(rhs.from))
        {
            return Some(*self.from);
        }

        if (self.to == rhs.from && !contains(rhs.to)) || (self.to == rhs.to && !contains(rhs.from))
        {
            return Some(*self.to);
        }

        None
    }
}

/// Represents a closed shape in the plain.
struct Polygon<T = f64> {
    /// The ordered list of vertices describing the polygon.  
    vertices: Vec<Point<T>>,
}

impl<T, P> From<Vec<P>> for Polygon<T>
where
    P: Into<Point<T>>,
{
    fn from(vertices: Vec<P>) -> Self {
        Self {
            vertices: vertices.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T> Polygon<T>
where
    T: Signed + Float,
{
    /// Returns the amount of times self winds around the given [`Point`].
    fn winding(&self, point: &Point<T>) -> isize {
        // Returns true if, and only if, the point is on the left of the infinite line containing
        // the given segment.
        let left_of = |segment: &Segment<'_, T>| {
            Determinant::from((segment, point))
                .into_inner()
                .is_positive()
        };

        self.segments().fold(0, |wn, segment| {
            if segment.from.y <= point.y && segment.to.y > point.y && left_of(&segment) {
                wn + 1
            } else if segment.from.y > point.y && segment.to.y <= point.y && !left_of(&segment) {
                wn - 1
            } else {
                wn
            }
        })
    }

    /// Returns true if, and only if, self contains the given point.
    fn contains(&self, point: &Point<T>) -> bool {
        self.winding(point) != 0
    }

    /// Returns true if, and only if, the polygon is oriented clockwise.
    fn is_clockwise(&self) -> bool {
        self.vertices
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                match a.y.partial_cmp(&b.y) {
                    Some(Ordering::Equal) => b.x.partial_cmp(&a.x),
                    other => other,
                }
                .unwrap_or(Ordering::Equal)
            })
            .map(|(mut position, min)| {
                // Avoids usize overflow when position = 0.
                position += self.vertices.len();

                Determinant::from([
                    &self.vertices[(position - 1) % self.vertices.len()],
                    min,
                    &self.vertices[(position + 1) % self.vertices.len()],
                ])
                .into_inner()
                .is_negative()
            })
            .unwrap_or_default()
    }
}

impl<T> Polygon<T> {
    /// Returns an ordered iterator over all the vertices of the polygon.
    ///
    /// By definition, a polygon is a closed shape, hence the latest point of the iterator equals
    /// the very first.
    fn vertices(&self) -> impl Iterator<Item = &Point<T>> {
        self.vertices.iter().chain(self.vertices.first())
    }

    /// Returns an ordered iterator over all the [`Segment`]s of this polygon.
    fn segments(&self) -> impl Iterator<Item = Segment<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(Segment::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        point::{point, Point},
        polygon::{Polygon, Segment},
    };

    #[test]
    fn segment_intersection() {
        struct Test {
            name: &'static str,
            segment: Segment<'static, f64>,
            rhs: Segment<'static, f64>,
            want: Option<Point>,
        }

        vec![
            Test {
                name: "Crossing segments",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(0., 4.),
                    to: &point!(4., 0.),
                },
                want: Some(point!(2., 2.)),
            },
            Test {
                name: "Segments starting at the same point",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(0., 0.),
                    to: &point!(-4., 4.),
                },
                want: Some(point!(0., 0.)),
            },
            Test {
                name: "Connected segments",
                segment: Segment {
                    from: &point!(4., 4.),
                    to: &point!(0., 0.),
                },
                rhs: Segment {
                    from: &point!(0., 0.),
                    to: &point!(-4., 4.),
                },
                want: Some(point!(0., 0.)),
            },
            Test {
                name: "Collinear segments with common point",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(-4., -4.),
                    to: &point!(0., 0.),
                },
                want: Some(point!(0., 0.)),
            },
            Test {
                name: "Collinear segments with no common point",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(-4., -4.),
                    to: &point!(-2., -2.),
                },
                want: None,
            },
            Test {
                name: "Segments ending at the same point",
                segment: Segment {
                    from: &point!(4., 4.),
                    to: &point!(0., 0.),
                },
                rhs: Segment {
                    from: &point!(-4., 4.),
                    to: &point!(0., 0.),
                },
                want: None,
            },
            Test {
                name: "Parallel segments",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(0., 4.),
                    to: &point!(4., 8.),
                },
                want: None,
            },
            Test {
                name: "Coincident segments",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(0., 0.),
                    to: &point!(2., 2.),
                },
                want: None,
            },
            Test {
                name: "Non-crossing segments",
                segment: Segment {
                    from: &point!(4., 4.),
                    to: &point!(8., 8.),
                },
                rhs: Segment {
                    from: &point!(0., 4.),
                    to: &point!(4., 0.),
                },
                want: None,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.segment.intersection(&test.rhs);
            assert_eq!(
                got, test.want,
                "{}: got intersection point = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }

    #[test]
    fn polygon_winding_number() {
        struct Test {
            name: &'static str,
            polygon: Polygon,
            point: Point,
            want: isize,
        }

        vec![
            Test {
                name: "Center of a counterclockwise polygon",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                point: [2., 2.].into(),
                want: 1,
            },
            Test {
                name: "Center of a clockwise polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [2., 2.].into(),
                want: -1,
            },
            Test {
                name: "On the left of the polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [-2., -2.].into(),
                want: 0,
            },
            Test {
                name: "On the right of the polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [6., 6.].into(),
                want: 0,
            },
            Test {
                name: "Inside self-crossing polygon",
                polygon: vec![
                    [8., 0.],
                    [8., 6.],
                    [2., 6.],
                    [2., 4.],
                    [6., 4.],
                    [6., 2.],
                    [4., 2.],
                    [4., 8.],
                    [0., 8.],
                    [0., 0.],
                ]
                .into(),
                point: [3., 5.].into(),
                want: 2,
            },
            Test {
                name: "Outside self-crossing polygon",
                polygon: vec![
                    [8., 0.],
                    [8., 6.],
                    [2., 6.],
                    [2., 4.],
                    [6., 4.],
                    [6., 2.],
                    [4., 2.],
                    [4., 8.],
                    [0., 8.],
                    [0., 0.],
                ]
                .into(),
                point: [5., 3.].into(),
                want: 0,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.polygon.winding(&test.point);
            assert_eq!(
                got, test.want,
                "{}: got winding number = {got}, want = {}",
                test.name, test.want
            );
        });
    }

    #[test]
    fn polygon_clockwise_orientation() {
        struct Test {
            name: &'static str,
            polygon: Polygon,
            want: bool,
        }

        vec![
            Test {
                name: "Simple counter-clockwise polygon",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                want: false,
            },
            Test {
                name: "Simple clockwise polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                want: true,
            },
            Test {
                name: "Self-crossing counter-clockwise polygon",
                polygon: vec![
                    [8., 0.],
                    [8., 6.],
                    [2., 6.],
                    [2., 4.],
                    [6., 4.],
                    [6., 2.],
                    [4., 2.],
                    [4., 8.],
                    [0., 8.],
                    [0., 0.],
                ]
                .into(),
                want: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.polygon.is_clockwise();
            assert_eq!(
                got, test.want,
                "{}: got is clockwise = {got}, want = {}",
                test.name, test.want
            );
        });
    }
}

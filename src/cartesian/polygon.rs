use std::{cmp::Ordering, fmt::Debug};

use num_traits::{Float, Signed};

use super::{
    determinant::Determinant,
    point::{Point, point},
};

/// Represents the straight line between two consecutive vertices of a [`Polygon`].
#[derive(Debug)]
pub struct Segment<'a, T> {
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
    T: Signed + Float,
{
    /// Returns the distance between the two endpoints of the segment.
    pub(crate) fn length(&self) -> T {
        self.from.distance(self.to)
    }

    /// Returns true if, and only if, the given [`Point`] exists within the segment.
    pub(crate) fn contains(&self, point: &Point<T>) -> bool {
        Determinant::from((self, point)).into_inner().is_zero()
            && self.from.distance(point) <= self.length()
            && self.to.distance(point) <= self.length()
    }

    /// Returns the [`Point`] of intersection between self and the given segment, if any.
    pub(crate) fn intersection(&self, rhs: &Self) -> Option<Point<T>> {
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

    /// Being zero the determinant of self and rhs, returns the single common [`Point`] between
    /// them, if any.
    fn collinear_common_point(&self, rhs: &Segment<'_, T>) -> Option<Point<T>> {
        let contains = |segment: &Segment<'_, T>, point| {
            // Optimized version of `Self::contains`:
            // No need to compute the cross product, by sharing an endpoint its guaranteed both
            // parallel segments are collinear.
            segment.from.distance(point) <= segment.length()
                && segment.to.distance(point) <= segment.length()
        };

        if (self.from == rhs.from && !contains(self, rhs.to) && !contains(rhs, self.to))
            || (self.from == rhs.to && !contains(self, rhs.from) && !contains(rhs, self.to))
        {
            return Some(*self.from);
        }

        if (self.to == rhs.from && !contains(self, rhs.to) && !contains(rhs, self.from))
            || (self.to == rhs.to && !contains(self, rhs.from) && !contains(rhs, self.from))
        {
            return Some(*self.to);
        }

        None
    }

    /// Returns the middle point between the endpoints of this segment.
    pub(crate) fn midpoint(&self) -> Point<T> {
        let two = T::one() + T::one();
        Point {
            x: (self.from.x + self.to.x) / two,
            y: (self.from.y + self.to.y) / two,
        }
    }
}

/// Represents a polygon in the plain.
#[derive(Debug, Clone)]
pub struct Polygon<T> {
    /// The ordered list of vertices describing the polygon.  
    pub(crate) vertices: Vec<Point<T>>,
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

impl<T> PartialEq for Polygon<T>
where
    T: Clone + PartialEq,
{
    /// Two polygons are equal if, and only if, they have the same vertices describing the same
    /// boundary.
    fn eq(&self, other: &Self) -> bool {
        let len = self.vertices.len();
        if len != other.vertices.len() {
            return false;
        }

        let mut double = other.vertices.clone();
        double.extend_from_slice(&other.vertices);

        let is_rotation = |double: &[Point<T>]| {
            (0..len).any(|padding| double[padding..padding + len] == self.vertices)
        };

        if is_rotation(&double) {
            return true;
        }

        double.reverse();
        is_rotation(&double)
    }
}

impl<T> Polygon<T>
where
    T: Signed + Float,
{
    /// Returns the amount of times self winds around the given [`Point`].
    pub(crate) fn winding(&self, point: &Point<T>) -> isize {
        // Returns true if, and only if, the point is on the left of the infinite line containing
        // the given segment.
        let left_of = |segment: &Segment<'_, T>| {
            Determinant::from((segment, point))
                .into_inner()
                .is_positive()
        };

        self.segments().fold(0, |wn, segment| {
            if segment.contains(point)
                || segment.from.y <= point.y && segment.to.y > point.y && left_of(&segment)
            {
                wn + 1
            } else if segment.from.y > point.y && segment.to.y <= point.y && !left_of(&segment) {
                wn - 1
            } else {
                wn
            }
        })
    }

    /// Returns true if, and only if, the polygon is oriented clockwise.
    pub(crate) fn is_clockwise(&self) -> bool {
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
    pub fn vertices(&self) -> impl Iterator<Item = &Point<T>> {
        self.vertices.iter().chain(self.vertices.first())
    }

    /// Returns an ordered iterator over all the [`Segment`]s of this polygon.
    pub fn segments(&self) -> impl Iterator<Item = Segment<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(Segment::from)
    }

    /// Returns a new polygon with the reverses order of vertices.
    pub(crate) fn reversed(mut self) -> Self {
        self.vertices.reverse();
        self
    }
}

/// The smallest rectangular box that completely encloses a [`Polygon`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BoundingBox<T> {
    /// The point containing the left-bottom coordinates.
    min: Point<T>,
    /// The point containing the right-top coordinates.
    max: Point<T>,
}

impl<T> From<&Polygon<T>> for BoundingBox<T>
where
    T: Float,
{
    /// Returns the smallest rectangular box that completely encloses the given [`Polygon`].
    fn from(polygon: &Polygon<T>) -> Self {
        polygon.vertices.iter().fold(
            Self {
                min: point!(T::infinity(), T::infinity()),
                max: point!(T::neg_infinity(), T::neg_infinity()),
            },
            |bb, vertex| Self {
                min: point!(bb.min.x.min(vertex.x), bb.min.y.min(vertex.y)),
                max: point!(bb.max.x.max(vertex.x), bb.max.y.max(vertex.y)),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::cartesian::{
        point::{Point, point},
        polygon::{BoundingBox, Polygon, Segment},
    };

    #[test]
    fn segment_intersection() {
        struct Test {
            name: &'static str,
            segment: Segment<'static, f64>,
            rhs: Segment<'static, f64>,
            want: Option<Point<f64>>,
        }

        vec![
            Test {
                name: "crossing segments",
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
                name: "segments starting at the same point",
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
                name: "connected segments",
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
                name: "collinear segments with common point",
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
                name: "collinear segments with no common point",
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
                name: "segments ending at the same point",
                segment: Segment {
                    from: &point!(4., 4.),
                    to: &point!(0., 0.),
                },
                rhs: Segment {
                    from: &point!(-4., 4.),
                    to: &point!(0., 0.),
                },
                want: Some(point!(0., 0.)),
            },
            Test {
                name: "parallel segments",
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
                name: "coincident segments when rhs is shorter",
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
                name: "coincident segments when rhs is larger",
                segment: Segment {
                    from: &point!(0., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(0., 0.),
                    to: &point!(8., 8.),
                },
                want: None,
            },
            Test {
                name: "non-crossing segments",
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
            Test {
                name: "perpendicular segments",
                segment: Segment {
                    from: &point!(4., 0.),
                    to: &point!(4., 4.),
                },
                rhs: Segment {
                    from: &point!(2., 2.),
                    to: &point!(6., 2.),
                },
                want: Some(point!(4., 2.)),
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
            polygon: Polygon<f64>,
            point: Point<f64>,
            want: isize,
        }

        vec![
            Test {
                name: "center of a counterclockwise polygon",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                point: [2., 2.].into(),
                want: 1,
            },
            Test {
                name: "center of a clockwise polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [2., 2.].into(),
                want: -1,
            },
            Test {
                name: "on the left of the polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [-2., -2.].into(),
                want: 0,
            },
            Test {
                name: "on the right of the polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [6., 6.].into(),
                want: 0,
            },
            Test {
                name: "inside self-crossing polygon",
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
                name: "outside self-crossing polygon",
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
            polygon: Polygon<f64>,
            want: bool,
        }

        vec![
            Test {
                name: "simple counter-clockwise polygon",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                want: false,
            },
            Test {
                name: "simple clockwise polygon",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                want: true,
            },
            Test {
                name: "self-crossing counter-clockwise polygon",
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

    #[test]
    fn polygon_bounding_box() {
        struct Test {
            name: &'static str,
            polygon: Polygon<f64>,
            want: BoundingBox<f64>,
        }

        vec![
            Test {
                name: "triangle",
                polygon: vec![[4., 0.], [4., 4.], [0., 0.]].into(),
                want: BoundingBox {
                    min: point!(0., 0.),
                    max: point!(4., 4.),
                },
            },
            Test {
                name: "rectangle",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                want: BoundingBox {
                    min: point!(0., 0.),
                    max: point!(4., 4.),
                },
            },
            Test {
                name: "self-crossing polygon",
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
                want: BoundingBox {
                    min: point!(0., 0.),
                    max: point!(8., 8.),
                },
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = BoundingBox::from(&test.polygon);
            assert_eq!(
                got, test.want,
                "{}: got bounding box = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }

    #[test]
    fn polygon_equaliry() {
        struct Test {
            name: &'static str,
            left: Polygon<f64>,
            right: Polygon<f64>,
            want: bool,
        }

        vec![
            Test {
                name: "same polygon",
                left: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                right: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                want: true,
            },
            Test {
                name: "with different orientation",
                left: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                right: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                want: true,
            },
            Test {
                name: "starting at different vertex",
                left: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                right: vec![[4., 4.], [0., 4.], [0., 0.], [4., 0.]].into(),
                want: true,
            },
            Test {
                name: "different polygons",
                left: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                right: vec![[4., 0.], [4., 4.], [0., 4.], [1., 1.]].into(),
                want: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.left == test.right;
            assert_eq!(
                got, test.want,
                "{}: got = {got}, want = {}",
                test.name, test.want
            );
        });
    }
}

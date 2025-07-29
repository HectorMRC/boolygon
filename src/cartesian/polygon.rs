use std::cmp::Ordering;

use num_traits::{Float, Signed};

use crate::{
    cartesian::{determinant::Determinant, Point, Segment},
    clipper::Operands,
    Geometry, RightHanded, Tolerance,
};

/// A polygon in the plain.
#[derive(Debug, Clone)]
pub struct Polygon<T> {
    /// The ordered list of vertices describing the polygon.  
    pub vertices: Vec<Point<T>>,
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

impl<T> RightHanded for Polygon<T>
where
    T: Signed + Float,
{
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

impl<T> Geometry for Polygon<T>
where
    T: Signed + Float,
{
    type Vertex = Point<T>;
    type Edge<'a>
        = Segment<'a, T>
    where
        Self: 'a;

    fn from_raw(_: Operands<Self>, vertices: Vec<Self::Vertex>, _: &Tolerance<T>) -> Option<Self> {
        Some(vertices.into())
    }

    fn total_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn edges(&self) -> impl Iterator<Item = Segment<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(|(from, to)| Segment { from, to })
    }

    fn reversed(mut self) -> Self {
        self.vertices.reverse();
        self
    }

    fn winding(&self, point: &Point<T>, _: &Tolerance<T>) -> isize {
        // Returns true if, and only if, the point is on the left of the infinite line containing
        // the given segment.
        let left_of = |segment: &Segment<'_, T>| {
            Determinant::from((segment, point))
                .into_inner()
                .is_positive()
        };

        self.edges().fold(0, |wn, segment| {
            if segment.from.y <= point.y && segment.to.y > point.y && left_of(&segment) {
                wn + 1
            } else if segment.from.y > point.y && segment.to.y <= point.y && !left_of(&segment) {
                wn - 1
            } else {
                wn
            }
        })
    }
}

impl<T> IntoIterator for Polygon<T> {
    type Item = Point<T>;
    type IntoIter = std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
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
}

/// A constructor macro for cartesian [`Polygon`]s.
#[macro_export]
macro_rules! cartesian_polygon {
    ($($vertices:expr),*) => {
        Polygon::from(vec![$($vertices),*])
    };
}

pub use cartesian_polygon;

#[cfg(test)]
mod tests {
    use crate::{
        cartesian::{point::Point, Polygon},
        Geometry, RightHanded,
    };

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
            let got = test.polygon.winding(&test.point, &Default::default());
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
    fn polygon_equality() {
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

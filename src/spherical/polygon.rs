use geocart::{
    transform::{Rotation, Transform},
    Cartesian,
};
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{clipper::Context, spherical::Arc, Edge, Geometry, Tolerance};

use super::Point;

/// A spherical polygon.
#[derive(Debug, Clone)]
pub struct Polygon<T> {
    /// The ordered list of vertices describing the polygon.  
    pub vertices: Vec<Point<T>>,
    /// A point outside this polygon.
    pub exterior: Point<T>,
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

        is_rotation(&double)
    }
}

impl<T> Geometry for Polygon<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    type Vertex = Point<T>;
    type Edge<'a>
        = Arc<'a, T>
    where
        Self: 'a;

    fn from_raw(
        ctx: Context<Self>,
        vertices: Vec<Self::Vertex>,
        tolerance: &Tolerance<T>,
    ) -> Option<Self> {
        let closest_exterior_point = |arc: &Arc<'_, T>, theta: T| {
            let midpoint = arc.midpoint().into();
            let normal = arc.normal();
            let tangent = normal.cross(&midpoint).normal();

            let candidate = Rotation::noop()
                .with_axis(tangent)
                .with_theta(theta.into())
                .transform(midpoint)
                .into();

            let subject_contains = ctx.operands.subject.contains(&candidate, tolerance);
            let clip_contains = ctx.operands.clip.contains(&candidate, tolerance);

            if !subject_contains && !clip_contains {
                return Some(candidate);
            }

            None
        };

        let mut exterior = None;
        let mut theta = T::PI() * tolerance.relative.into_inner();

        while exterior.is_none() && theta < T::FRAC_PI_8() {
            exterior = ctx
                .operands
                .subject
                .edges()
                .chain(ctx.operands.clip.edges())
                .find_map(|arc| {
                    closest_exterior_point(&arc, theta)
                        .or_else(|| closest_exterior_point(&arc, -theta))
                });

            theta = theta + theta;
        }

        exterior.map(|exterior| Self { vertices, exterior })
    }

    fn total_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn edges(&self) -> impl Iterator<Item = Arc<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(|(from, to)| Arc { from, to })
    }

    fn reversed(mut self) -> Self {
        self.vertices.reverse();
        self
    }

    fn winding(&self, point: &Point<T>, tolerance: &Tolerance<T>) -> isize {
        // Returns true if, and only if, the point is on the left of the great circle containing
        // the given arc.
        let left_of = |arc: &Arc<'_, T>| {
            let point = Cartesian::from(*point);
            arc.normal().dot(&point).is_positive()
        };

        self.edges()
            .filter(|segment| {
                Arc::new(&self.exterior, point)
                    .intersection(segment, tolerance)
                    .is_some()
            })
            .fold(0, |wn, arc| if left_of(&arc) { wn + 1 } else { wn - 1 })
    }

    fn is_clockwise(&self) -> bool {
        // Since the exterior point of the polygon is used as the observer, the actual orientation
        // is inverted. That implies that if the product of the polygon's normal and its exterior
        // is positive (counterclockwise from the observer's perspective), an observer inside
        // perceives the polygon's orientation as clockwise.

        self.edges()
            .fold(Cartesian::origin(), |normal, edge| {
                let from = Cartesian::from(*edge.from);
                let to = Cartesian::from(*edge.to);
                normal + from.cross(&to)
            })
            .dot(&self.exterior.into())
            > T::zero()
    }
}

impl<'a, T> IntoIterator for &'a Polygon<T> {
    type Item = &'a Point<T>;
    type IntoIter = std::slice::Iter<'a, Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.iter()
    }
}

impl<T> Polygon<T> {
    /// Returns a polygon with the given vertices and exterior.
    pub fn new<U>(vertices: Vec<U>, exterior: U) -> Self
    where
        U: Into<Point<T>>,
    {
        Self {
            vertices: vertices.into_iter().map(Into::into).collect(),
            exterior: exterior.into(),
        }
    }

    /// Returns an ordered iterator over all the vertices of the polygon.
    ///
    /// By definition, a polygon is a closed shape, hence the latest point of the iterator equals
    /// the very first.
    fn vertices(&self) -> impl Iterator<Item = &Point<T>> {
        self.vertices.iter().chain(self.vertices.first())
    }
}

/// A constructor macro for the spherical [`Polygon`].
#[macro_export]
macro_rules! spherical_polygon {
    ($($vertices:expr),*; $exterior:expr) => {
        Polygon::new(vec![$($vertices),*], $exterior)
    };
}

pub use spherical_polygon;

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

    use crate::{
        spherical::{Point, Polygon},
        Geometry, Tolerance,
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
                polygon: spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                ),
                point: [FRAC_PI_4, FRAC_PI_4].into(),
                want: 1,
            },
            Test {
                name: "center of a clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.],
                    [0., 0.];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                ),
                point: [FRAC_PI_4, FRAC_PI_4].into(),
                want: -1,
            },
            Test {
                name: "on the left of the polygon",
                polygon: spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [3. * FRAC_PI_2, 0.];
                    [FRAC_PI_4, 3. * FRAC_PI_2]
                ),
                point: [FRAC_PI_2, 3. * FRAC_PI_2].into(),
                want: 0,
            },
            Test {
                name: "on the right of the polygon",
                polygon: spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [3. * FRAC_PI_2, 0.];
                    [FRAC_PI_4, FRAC_PI_2]
                ),
                point: [FRAC_PI_2, FRAC_PI_2].into(),
                want: 0,
            },
            Test {
                name: "inside self-crossing polygon",
                polygon: spherical_polygon![
                    [0., 0.],
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, FRAC_PI_4],
                    [FRAC_PI_4, 3. * FRAC_PI_2 + FRAC_PI_4],
                    [FRAC_PI_8, 0.],
                    [FRAC_PI_4, FRAC_PI_4],
                    [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                ]
                .into(),
                point: [FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                want: 2,
            },
            Test {
                name: "outside self-crossing polygon",
                polygon: spherical_polygon![
                    [0., 0.],
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, FRAC_PI_4],
                    [FRAC_PI_4, 3. * FRAC_PI_2 + FRAC_PI_4],
                    [FRAC_PI_8, 0.],
                    [FRAC_PI_4, FRAC_PI_4],
                    [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                ]
                .into(),
                point: [FRAC_PI_4, 0.].into(),
                want: 0,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.polygon.winding(&test.point, &tolerance);
            assert_eq!(got, test.want, "{}", test.name);
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
                name: "equator as a counter-clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                want: false,
            },
            Test {
                name: "equator as a clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.];
                    [PI, 0.]
                ),
                want: true,
            },
            Test {
                name: "spherical right triangle as a counter-clockwise polygon",
                polygon: spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, PI]
                ),
                want: false,
            },
            Test {
                name: "spherical right triangle as a clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.],
                    [0., 0.];
                    [PI, PI]
                ),
                want: true,
            },
            Test {
                name: "semi-corona",
                polygon: spherical_polygon!(
                    [FRAC_PI_4, 0.],
                    [FRAC_PI_4, 3. * FRAC_PI_2],
                    [FRAC_PI_4, PI],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, 0.];
                    [PI, 0.]
                ),
                want: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.polygon.is_clockwise();
            assert_eq!(got, test.want, "{}", test.name);
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
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                want: true,
            },
            Test {
                name: "starting at different vertex",
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI];
                    [PI, 0.]
                ),
                want: true,
            },
            Test {
                name: "with different orientation",
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.];
                    [PI, 0.]
                ),
                want: false,
            },
            Test {
                name: "different polygons",
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, PI];
                    [PI, 0.]
                ),
                want: false,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.left == test.right;
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

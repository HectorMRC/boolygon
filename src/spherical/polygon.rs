use std::cmp::Ordering;

use geocart::{
    transform::{Rotation, Transform},
    Cartesian,
};
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{
    clipper::Operands, spherical::Arc, Edge, FromRaw, Geometry, Midpoint, RightHanded, Secant,
    Tolerance, Wind,
};

use super::Point;

/// Represents a spherical polygon.
#[derive(Debug, Clone)]
pub struct Polygon<T> {
    /// The ordered list of vertices describing the polygon.  
    pub vertices: Vec<Point<T>>,
    /// A point within this polygon.
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

        if is_rotation(&double) {
            return true;
        }

        double.reverse();
        is_rotation(&double)
    }
}

impl<T> RightHanded for Polygon<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn is_clockwise(&self) -> bool {
        self.vertices
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                match a.polar_angle.partial_cmp(&b.polar_angle) {
                    Some(Ordering::Equal) => b.azimuthal_angle.partial_cmp(&a.azimuthal_angle),
                    other => other,
                }
                .unwrap_or(Ordering::Equal)
            })
            .map(|(mut position, &min)| {
                // Avoids usize overflow when position = 0.
                position += self.vertices.len();
                let before = Cartesian::from(self.vertices[(position - 1) % self.vertices.len()]);
                let after = Cartesian::from(self.vertices[(position + 1) % self.vertices.len()]);
                let min = Cartesian::from(min);

                before.dot(&min.cross(&after)).is_negative()
            })
            .unwrap_or_default()
    }
}

impl<T> Wind for Polygon<T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
{
    type Point = Point<T>;

    fn reversed(mut self) -> Self {
        self.vertices.reverse();
        self
    }

    fn winding(&self, point: &Self::Point, tolerance: &Tolerance<T>) -> isize {
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
            .fold(0, |wn, arc| {
                if left_of(&arc) {
                    wn + 1
                } else if !left_of(&arc) {
                    wn - 1
                } else {
                    wn
                }
            })
    }
}

impl<T> FromRaw for Polygon<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    fn from_raw(
        operands: Operands<Self>,
        vertices: Vec<Self::Point>,
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

            let subject_contains = operands.subject.contains(&candidate, tolerance);
            let clip_contains = operands.clip.contains(&candidate, tolerance);

            if !subject_contains && !clip_contains {
                return Some(candidate);
            }

            None
        };

        let theta = T::PI() * tolerance.relative.into_inner();

        operands
            .subject
            .edges()
            .chain(operands.clip.edges())
            .find_map(|arc| {
                closest_exterior_point(&arc, theta).or_else(|| closest_exterior_point(&arc, -theta))
            })
            .or_else(|| {
                operands
                    .subject
                    .polygons
                    .iter()
                    .map(|polygon| polygon.exterior)
                    .find(|exterior| !operands.clip.contains(&exterior, tolerance))
            })
            .or_else(|| {
                operands
                    .clip
                    .polygons
                    .iter()
                    .map(|polygon| polygon.exterior)
                    .find(|exterior| !operands.subject.contains(&exterior, tolerance))
            })
            .map(|exterior| Self { vertices, exterior })
    }
}

impl<T> Geometry for Polygon<T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    type Edge<'a>
        = Arc<'a, T>
    where
        Self: 'a;

    fn total_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn edges(&self) -> impl Iterator<Item = Arc<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(|(from, to)| Arc { from, to })
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
    /// Returns a polygon with the given vertices and exterior.
    pub fn new<U>(exterior: U, vertices: Vec<U>) -> Self
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
        Polygon::new($exterior, vec![$($vertices),*])
    };
}

pub use spherical_polygon;

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

    use crate::{
        spherical::{Point, Polygon},
        RightHanded, Tolerance, Wind,
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
                    [PI + FRAC_PI_2, 0.];
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
                    [PI + FRAC_PI_2, 0.];
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
                name: "equator as a counter-clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, PI + FRAC_PI_2];
                    [PI, 0.]
                ),
                want: false,
            },
            Test {
                name: "equator as a clockwise polygon",
                polygon: spherical_polygon!(
                    [FRAC_PI_2, PI + FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.];
                    [0., 0.]
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
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, PI + FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, PI + FRAC_PI_2];
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
                    [FRAC_PI_2, PI + FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, PI + FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, 0.];
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
                    [FRAC_PI_2, PI + FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, PI + FRAC_PI_2],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI];
                    [PI, 0.]
                ),
                want: true,
            },
            Test {
                name: "different polygons",
                left: spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, PI + FRAC_PI_2];
                    [PI, 0.]
                ),
                right: spherical_polygon!(
                    [FRAC_PI_2, PI + FRAC_PI_2],
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
            assert_eq!(
                got, test.want,
                "{}: got = {got}, want = {}",
                test.name, test.want
            );
        });
    }

    // #[test]
    // fn shape_winding_number() {
    //     struct Test {
    //         name: &'static str,
    //         shape: Shape<Polygon<f64>>,
    //         point: Point<f64>,
    //         want: isize,
    //     }

    //     vec![Test {
    //         name: "outside shape with hole",
    //         shape: Shape {
    //             polygons: vec![
    //                 spherical_polygon!(
    //                     [FRAC_PI_2, 0.],
    //                     [FRAC_PI_2, FRAC_PI_2],
    //                     [FRAC_PI_2, PI],
    //                     [FRAC_PI_2, 3. * FRAC_PI_2];
    //                     [PI, 0.]
    //                 ),
    //                 spherical_polygon!(
    //                     [FRAC_PI_4, 3. * FRAC_PI_2],
    //                     [FRAC_PI_4, PI],
    //                     [FRAC_PI_4, FRAC_PI_2],
    //                     [FRAC_PI_4, 0.];
    //                     [PI, 0.]
    //                 ),
    //             ],
    //         },
    //         point: [0., 0.].into(),
    //         want: 0,
    //     }]
    //     .into_iter()
    //     .for_each(|test| {
    //         let tolerance = Tolerance {
    //             relative: 1e-09.into(),
    //             absolute: 0.0.into(),
    //         };

    //         let got = test.shape.winding(&test.point, &tolerance);
    //         assert_eq!(
    //             got, test.want,
    //             "{}: got winding number = {got}, want = {}",
    //             test.name, test.want
    //         );
    //     });
    // }
}

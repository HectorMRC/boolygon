use geocart::Cartesian;
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{spherical::Point, Edge, Neighbors, MaybePair, IntersectionKind, IsClose, Tolerance, Vertex as _};

/// The undirected arc between two endpoints.
#[derive(Debug)]
pub struct Arc<'a, T> {
    /// The first point in the segment.
    pub(crate) from: &'a Point<T>,
    /// The last point in the segment.
    pub(crate) to: &'a Point<T>,
}

impl<'a, T> Edge<'a> for Arc<'a, T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    type Vertex = Point<T>;

    fn new(from: &'a Self::Vertex, to: &'a Self::Vertex) -> Self {
        Self { from, to }
    }

    fn midpoint(&self) -> Self::Vertex {
        if self.is_antipodal() {
            return Point {
                inclination: (T::FRAC_PI_2() + self.from.inclination.into_inner()).into(),
                azimuth: (T::FRAC_PI_2() + self.from.azimuth.into_inner()).into(),
            };
        }

        (Cartesian::from(*self.from) + Cartesian::from(*self.to))
            .normal()
            .into()
    }

    fn contains(&self, point: &Self::Vertex, tolerance: &Tolerance<T>) -> bool {
        let from_distance = self.from.distance(point);
        let to_distance = self.to.distance(point);
        let total_length = from_distance + to_distance;
        let actual_lenght = self.length();

        total_length.is_close(&actual_lenght, tolerance)
    }

    fn intersection(
        &self,
        other: &Self,
        tolerance: &Tolerance<T>,
    ) -> Option<MaybePair<Self::Vertex>> {
        if self.is_antipodal() {
            let point = self.midpoint();
            let first_half = Arc::new(self.from, &point);
            let second_half = Arc::new(&point, self.to);

            let Some(first_intersection) = other.intersection(&first_half, tolerance) else {
                return other.intersection(&second_half, tolerance);
            };

            if matches!(first_intersection, MaybePair::Pair(_)) {
                return Some(first_intersection);
            }

            let Some(second_intersection) = other.intersection(&second_half, tolerance) else {
                return Some(first_intersection);
            };

            return match (first_intersection, second_intersection) {
                (MaybePair::Single(start), MaybePair::Single(end)) => Some(MaybePair::Pair([start, end])),
                (_, intersection_range) => Some(intersection_range),
            };
        }

        let direction = self.normal().cross(&other.normal());
        if direction.magnitude().is_close(&T::zero(), tolerance) {
            // When two arcs lie on the same great circle, their normal vectors coincide.
            return self.co_great_circular_common_points(other, tolerance);
        }

        // TODO: remove this if statements
        if self.contains(other.from, tolerance) {
            return Some(MaybePair::Single(*other.from));
        }

        if self.contains(other.to, tolerance) {
            return Some(MaybePair::Single(*other.to));
        }

        if other.contains(self.from, tolerance) {
            return Some(MaybePair::Single(*self.from));
        }

        if other.contains(self.to, tolerance) {
            return Some(MaybePair::Single(*self.to));
        }

        let lambda = T::one() / direction.magnitude();

        let intersection = (direction * lambda).into();
        if self.contains(&intersection, tolerance) && other.contains(&intersection, tolerance) {
            return Some(MaybePair::Single(intersection));
        }

        let intersection = (direction * -lambda).into();
        if self.contains(&intersection, tolerance) && other.contains(&intersection, tolerance) {
            return Some(MaybePair::Single(intersection));
        }

        None
    }

    fn intersection_kind(
        _intersection: &'a Self::Vertex,
        _neighbors: Neighbors<'a, Self::Vertex>,
        _sibling_neighbors: Neighbors<'a, Self::Vertex>,
        _tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> IntersectionKind {
        todo!()
    }

    fn side(&self, _point: &Self::Vertex) -> Option<crate::Side> {
        todo!()
    }
}

impl<T> Arc<'_, T>
where
    T: PartialOrd + Signed + Float + FloatConst + Euclid,
{
    /// Returns the normal vector of the great circle containing the endpoints of self.
    pub(crate) fn normal(&self) -> Cartesian<T> {
        let from = Cartesian::from(*self.from);
        let to = Cartesian::from(*self.to);
        from.cross(&to).normal()
    }

    /// Being self and the other two arcs lying on the same great circle, returns the
    /// intersection between them, if any.
    fn co_great_circular_common_points(
        &self,
        other: &Self,
        tolerance: &Tolerance<T>,
    ) -> Option<MaybePair<Point<T>>> {
        let self_containement = (
            self.contains(other.from, tolerance),
            self.contains(other.to, tolerance),
        );

        if let (true, true) = self_containement {
            return Some(MaybePair::Pair([*other.from, *other.to]));
        }

        let other_containement = (
            other.contains(self.from, tolerance),
            other.contains(self.to, tolerance),
        );

        match (self_containement, other_containement) {
            (_, (true, true)) => Some(MaybePair::Pair([*self.from, *self.to])),
            ((true, _), (_, true)) => Some(if other.from != self.to {
                MaybePair::Pair([*other.from, *self.to])
            } else {
                MaybePair::Single(*self.to)
            }),
            ((true, _), (true, _)) => Some(if other.from != self.from {
                MaybePair::Pair([*other.from, *self.from])
            } else {
                MaybePair::Single(*self.from)
            }),
            ((_, true), (true, _)) => Some(if other.to != self.from {
                MaybePair::Pair([*other.to, *self.from])
            } else {
                MaybePair::Single(*self.from)
            }),
            ((_, true), (_, true)) => Some(if other.to != self.to {
                MaybePair::Pair([*other.to, *self.to])
            } else {
                MaybePair::Single(*self.to)
            }),
            _ => None,
        }
    }
}

impl<T> Arc<'_, T>
where
    T: Signed + Float + FloatConst + Euclid,
{
    /// Returns the distance between the two endpoints of this arc.
    fn length(&self) -> T {
        self.from.distance(self.to)
    }

    /// Returns true if, and only if, the endpoints in the arc are antipodals.
    fn is_antipodal(&self) -> bool {
        let from = Cartesian::from(*self.from);
        let to = Cartesian::from(*self.to);
        from.dot(&to) == -T::one()
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

    use crate::{
        spherical::{Arc, Point}, Edge, MaybePair, Tolerance
    };

    #[test]
    fn arc_intersection() {
        struct Test<'a> {
            name: &'a str,
            arc: Arc<'a, f64>,
            other: Arc<'a, f64>,
            want: Option<MaybePair<Point<f64>>>,
        }

        vec![
            Test {
                name: "non-crossing arcs",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_4, 0.].into(),
                    to: &[FRAC_PI_4, FRAC_PI_2].into(),
                },
                want: None,
            },
            Test {
                name: "perpendicular with no common point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_2].into(),
                    to: &[FRAC_PI_2, PI].into(),
                },
                want: None,
            },
            Test {
                name: "perpendicular with endpoint in line",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4].into(),
                    to: &[FRAC_PI_2, FRAC_PI_4].into(),
                },
                want: Some(MaybePair::Single([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "perpendicular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(MaybePair::Single([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "perpendicular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(MaybePair::Single([0., 0.].into())),
            },
            Test {
                name: "perpendicular arcs ending at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_2].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(MaybePair::Single([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "co-great-circular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, PI].into(),
                },
                want: Some(MaybePair::Single([0., 0.].into())),
            },
            Test {
                name: "co-great-circular arcs ending at the same point",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, PI].into(),
                    to: &[PI, 0.].into(),
                },
                want: Some(MaybePair::Single([PI, 0.].into())),
            },
            Test {
                name: "co-great-circular arcs with no common point",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, PI].into(),
                    to: &[0., 0.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident arcs when other is shorter",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_4, 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_4, 0.].into(),
                    [FRAC_PI_2, 0.].into(),
                ])),
            },
            Test {
                name: "coincident arcs when other is larger",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_4].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_2, 0.].into(),
                    [FRAC_PI_2, FRAC_PI_4].into(),
                ])),
            },
            Test {
                name: "coincident arcs when arc contains other",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_8, 0.].into(),
                    to: &[FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_8, 0.].into(),
                    [FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                ])),
            },
            Test {
                name: "coincident arcs when other contains arc",
                arc: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_8].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2 - FRAC_PI_8].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_2, FRAC_PI_8].into(),
                    [FRAC_PI_2, FRAC_PI_2 - FRAC_PI_8].into(),
                ])),
            },
            Test {
                name: "coincident arcs when none is fully contained",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2 + FRAC_PI_4, 0.].into(),
                },
                other: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_2, 0.].into(),
                    [FRAC_PI_2 + FRAC_PI_4, 0.].into(),
                ])),
            },
            Test {
                name: "coincident at oposite direction when none is fully contained",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2 + FRAC_PI_4, 0.].into(),
                },
                other: Arc {
                    from: &[PI, 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(MaybePair::Pair([
                    [FRAC_PI_2, 0.].into(),
                    [FRAC_PI_2 + FRAC_PI_4, 0.].into(),
                ])),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.arc.intersection(&test.other, &tolerance);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

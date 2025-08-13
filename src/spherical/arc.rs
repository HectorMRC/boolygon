use geocart::Cartesian;
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{either::Either, spherical::Point, Edge, IsClose, Tolerance, Vertex as _};

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
        rhs: &Self,
        tolerance: &Tolerance<T>,
    ) -> Option<Either<Self::Vertex, [Self::Vertex; 2]>> {
        if self.is_antipodal() {
            let point = self.midpoint();
            let first_half = Arc::new(self.from, &point);
            let second_half = Arc::new(&point, self.to);

            let Some(first_intersection) = rhs.intersection(&first_half, tolerance) else {
                return rhs.intersection(&second_half, tolerance);
            };

            if first_intersection.is_right() {
                return Some(first_intersection);
            }

            let Some(second_intersection) = rhs.intersection(&second_half, tolerance) else {
                return Some(first_intersection);
            };

            return match (first_intersection, second_intersection) {
                (Either::Left(start), Either::Left(end)) => Some(Either::Right([start, end])),
                (_, intersection_range) => Some(intersection_range),
            };
        }

        let direction = self.normal().cross(&rhs.normal());
        if direction.magnitude().is_close(&T::zero(), tolerance) {
            // When two arcs lie on the same great circle, their normal vectors coincide.
            return self.co_great_circular_common_points(rhs, tolerance);
        }

        // TODO: remove this if statements
        if self.contains(rhs.from, tolerance) {
            return Some(Either::Left(*rhs.from));
        }

        if self.contains(rhs.to, tolerance) {
            return Some(Either::Left(*rhs.to));
        }

        if rhs.contains(self.from, tolerance) {
            return Some(Either::Left(*self.from));
        }

        if rhs.contains(self.to, tolerance) {
            return Some(Either::Left(*self.to));
        }

        let lambda = T::one() / direction.magnitude();

        let intersection = (direction * lambda).into();
        if self.contains(&intersection, tolerance) && rhs.contains(&intersection, tolerance) {
            return Some(Either::Left(intersection));
        }

        let intersection = (direction * -lambda).into();
        if self.contains(&intersection, tolerance) && rhs.contains(&intersection, tolerance) {
            return Some(Either::Left(intersection));
        }

        None
    }

    fn start(&self) -> &Self::Vertex {
        self.from
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

    /// Being self and rhs two arcs lying on the same great circle, returns the intersection
    /// between them, if any.
    fn co_great_circular_common_points(
        &self,
        rhs: &Self,
        tolerance: &Tolerance<T>,
    ) -> Option<Either<Point<T>, [Point<T>; 2]>> {
        let self_containement = (
            self.contains(&rhs.from, tolerance),
            self.contains(&rhs.to, tolerance),
        );

        if let (true, true) = self_containement {
            return Some(Either::Right([*rhs.from, *rhs.to]));
        }

        let rhs_containement = (
            rhs.contains(&self.from, tolerance),
            rhs.contains(&self.to, tolerance),
        );

        match (self_containement, rhs_containement) {
            (_, (true, true)) => Some(Either::Right([*self.from, *self.to])),
            ((true, _), (_, true)) => Some(if rhs.from != self.to {
                Either::Right([*rhs.from, *self.to])
            } else {
                Either::Left(*self.to)
            }),
            ((true, _), (true, _)) => Some(if rhs.from != self.from {
                Either::Right([*rhs.from, *self.from])
            } else {
                Either::Left(*self.from)
            }),
            ((_, true), (true, _)) => Some(if rhs.to != self.from {
                Either::Right([*rhs.to, *self.from])
            } else {
                Either::Left(*self.from)
            }),
            ((_, true), (_, true)) => Some(if rhs.to != self.to {
                Either::Right([*rhs.to, *self.to])
            } else {
                Either::Left(*self.to)
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
        either::Either,
        spherical::{Arc, Point},
        Edge, Tolerance,
    };

    #[test]
    fn arc_intersection() {
        struct Test<'a> {
            name: &'a str,
            arc: Arc<'a, f64>,
            rhs: Arc<'a, f64>,
            want: Option<Either<Point<f64>, [Point<f64>; 2]>>,
        }

        vec![
            Test {
                name: "non-crossing arcs",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                rhs: Arc {
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
                rhs: Arc {
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
                rhs: Arc {
                    from: &[FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4].into(),
                    to: &[FRAC_PI_2, FRAC_PI_4].into(),
                },
                want: Some(Either::Left([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "perpendicular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(Either::Left([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "perpendicular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(Either::Left([0., 0.].into())),
            },
            Test {
                name: "perpendicular arcs ending at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_2].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(Either::Left([FRAC_PI_2, 0.].into())),
            },
            Test {
                name: "co-great-circular arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, PI].into(),
                },
                want: Some(Either::Left([0., 0.].into())),
            },
            Test {
                name: "co-great-circular arcs ending at the same point",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, PI].into(),
                    to: &[PI, 0.].into(),
                },
                want: Some(Either::Left([PI, 0.].into())),
            },
            Test {
                name: "co-great-circular arcs with no common point",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, PI].into(),
                    to: &[0., 0.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident arcs when rhs is shorter",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_4, 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(Either::Right([
                    [FRAC_PI_4, 0.].into(),
                    [FRAC_PI_2, 0.].into(),
                ])),
            },
            Test {
                name: "coincident arcs when rhs is larger",
                arc: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_4].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(Either::Right([
                    [FRAC_PI_2, 0.].into(),
                    [FRAC_PI_2, FRAC_PI_4].into(),
                ])),
            },
            Test {
                name: "coincident arcs when arc contains rhs",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_8, 0.].into(),
                    to: &[FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                },
                want: Some(Either::Right([
                    [FRAC_PI_8, 0.].into(),
                    [FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                ])),
            },
            Test {
                name: "coincident arcs when rhs contains arc",
                arc: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_8].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2 - FRAC_PI_8].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                want: Some(Either::Right([
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
                rhs: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[PI, 0.].into(),
                },
                want: Some(Either::Right([
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
                rhs: Arc {
                    from: &[PI, 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: Some(Either::Right([
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

            let got = test.arc.intersection(&test.rhs, &tolerance);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

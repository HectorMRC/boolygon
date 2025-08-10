use geocart::Cartesian;
use num_traits::{Euclid, Float, FloatConst, Signed};

use crate::{spherical::Point, Edge, IsClose, Tolerance, Vertex as _};

/// The arc between two endpoints.
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

    fn intersection(&self, rhs: &Self, tolerance: &Tolerance<T>) -> Option<Self::Vertex> {
        if self.is_antipodal() {
            let point = self.midpoint();
            let first_half = Arc::new(self.from, &point);
            let second_half = Arc::new(&point, self.to);

            return rhs
                .intersection(&first_half, tolerance)
                .or_else(|| rhs.intersection(&second_half, tolerance));
        }

        let direction = self.normal().cross(&rhs.normal());
         if direction.magnitude().is_zero() {
            // When two arcs lie on the same great circle, their normal vectors coincide.
            return self.co_great_circular_common_point(rhs, tolerance);
        }

        if self.contains(rhs.from, tolerance) {
            return Some(*rhs.from);
        }

        if self.contains(rhs.to, tolerance) {
            return Some(*rhs.to);
        }

        if rhs.contains(self.from, tolerance) {
            return Some(*self.from);
        }

        if rhs.contains(self.to, tolerance) {
            return Some(*self.to);
        }

        let exclusive_contains = |arc: &Arc<'_, T>, intersection: &Point<T>| {
            arc.contains(intersection, tolerance)
                // && !arc.from.is_close(intersection, tolerance) 
                // && !arc.to.is_close(intersection, tolerance)
        };

        let lambda = T::one() / direction.magnitude();

        let intersection = (direction * lambda).into();
        if exclusive_contains(self, &intersection)
            && exclusive_contains(rhs, &intersection) {
            return Some(intersection);
        }

        let intersection = (direction * -lambda).into();
        if exclusive_contains(self, &intersection)
            && exclusive_contains(rhs, &intersection) {
            return Some(intersection);
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
        from.normal().cross(&to.normal()).normal()
    }

    /// Being self and rhs two arcs lying on the same great circle, returns the single common
    /// [`Point`] between them, if any.
    fn co_great_circular_common_point(
        &self,
        rhs: &Self,
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
        spherical::{Arc, Point},
        Edge, Tolerance,
    };

    #[test]
    fn arc_intersection() {
        struct Test<'a> {
            name: &'a str,
            arc: Arc<'a, f64>,
            rhs: Arc<'a, f64>,
            want: Option<Point<f64>>,
        }

        vec![
            Test {
                name: "perpendicular arcs",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_4, 3. * FRAC_PI_2 + FRAC_PI_4].into(),
                    to: &[FRAC_PI_4, FRAC_PI_4].into(),
                },
                want: Some([0.61547970867038715, 0.].into()),
            },
            Test {
                name: "arcs starting at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                // want: None,
                want: Some([0., 0.].into())
            },
            Test {
                name: "connected arcs",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, 0.].into(),
                    to: &[FRAC_PI_2, FRAC_PI_2].into(),
                },
                // want: None,
                want: Some([FRAC_PI_2, 0.].into())
            },
            Test {
                name: "co-great-circular arcs with common point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[PI, 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                // want: None,
                want: Some([FRAC_PI_2, 0.].into())
            },
            Test {
                name: "co-great-circular arcs with no common point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[PI, 0.].into(),
                    to: &[FRAC_PI_2, -PI].into(),
                },
                want: None,
            },
            Test {
                name: "arcs ending at the same point",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, FRAC_PI_2].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                // want: None,
                want: Some([FRAC_PI_2, 0.].into()),
            },
            Test {
                name: "parallel arcs",
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
                name: "coincident arcs when rhs is shorter",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_4, 0.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident arcs when rhs is larger",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2 + FRAC_PI_4, 0.].into(),
                },
                want: None,
            },
            Test {
                name: "rhs inside arc",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_8, 0.].into(),
                    to: &[FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                },
                want: None,
            },
            Test {
                name: "arc inside rhs",
                arc: Arc {
                    from: &[FRAC_PI_8, 0.].into(),
                    to: &[FRAC_PI_2 - FRAC_PI_8, 0.].into(),
                },
                rhs: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                want: None,
            },
            Test {
                name: "non-crossing arcs",
                arc: Arc {
                    from: &[0., 0.].into(),
                    to: &[FRAC_PI_2, 0.].into(),
                },
                rhs: Arc {
                    from: &[FRAC_PI_2, PI].into(),
                    to: &[PI, 0.].into(),
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
                    from: &[FRAC_PI_4, 0.].into(),
                    to: &[FRAC_PI_4, FRAC_PI_4].into(),
                },
                // want: None,
                want: Some([FRAC_PI_4, 0.].into())
            },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.arc.intersection(&test.rhs, &tolerance);
            
            assert_eq!(
                got, test.want,
                "{}: got intersection point = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }
}

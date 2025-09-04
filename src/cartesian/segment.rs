use num_traits::{Float, FloatConst, Signed};

use crate::{
    cartesian::{determinant::Determinant, Point}, Corner, Edge, Event, IsClose, MaybePair, Side, Tolerance, Vertex
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
    T: Signed + Float + FloatConst,
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

    fn intersection(
        &self,
        other: &Self,
        _: &Tolerance<T>,
    ) -> Option<MaybePair<Self::Vertex>> {
        let determinant = self.determinant(other).into_inner();

        if determinant.is_zero() {
            // When the two (infinite) lines are parallel or coincident, the determinant is zero.
            return if self
                .determinant(&Segment::new(other.from, self.from))
                .into_inner()
                .is_zero()
            {
                self.collinear_common_points(other)
            } else {
                Default::default()
            };
        }

        let t = (self.from.x - other.from.x) * (other.from.y - other.to.y)
            - (self.from.y - other.from.y) * (other.from.x - other.to.x);

        let t = t / determinant;
        if !(T::zero()..=T::one()).contains(&t) {
            return Default::default();
        }

        let u = -((self.from.x - self.to.x) * (self.from.y - other.from.y)
            - (self.from.y - self.to.y) * (self.from.x - other.from.x));

        let u = u / determinant;
        if !(T::zero()..=T::one()).contains(&u) {
            return Default::default();
        }

        Some(MaybePair::Single(Point {
            x: self.from.x + t * (self.to.x - self.from.x),
            y: self.from.y + t * (self.to.y - self.from.y),
        }))
    }

    fn event(
        corner: Corner<'a, Point<T>>,
        tolerance: &Tolerance<T>,
    ) -> Option<Event> {
        let tail = Self::new(corner.neighbors.tail, corner.vertex);
        let head = Self::new(corner.vertex, &corner.neighbors.head);

        let sibling = corner.intersection.map(|intersection| intersection.neighbors)?;
        let sibling_tail = Segment::new(&sibling.tail, corner.vertex);
        let sibling_head = Segment::new(corner.vertex, &sibling.head);

        let overlap = |edge: &Self| {
            let midpoint = edge.midpoint();

            sibling_tail.contains(&midpoint, tolerance)
                || sibling_head.contains(&midpoint, tolerance)
                || edge.contains(&sibling_tail.midpoint(), tolerance)
                || edge.contains(&sibling_head.midpoint(), tolerance)
        };

        let angle = |other: &Self::Vertex| {
            let atan = (other.y - corner.vertex.y).atan2(other.x - corner.vertex.x);
            if atan < T::zero() {
                atan + T::TAU()
            } else {
                atan
            }
        };

        let inside = |alpha| {
            let sibling_tail_angle = angle(&sibling.tail);
            let sibling_head_angle = angle(&sibling.head);

            if sibling_tail_angle < sibling_head_angle {
                alpha < sibling_tail_angle || sibling_head_angle < alpha
            } else {
                sibling_head_angle < alpha && alpha < sibling_tail_angle
            }
        };

        let tail_is_inside = overlap(&tail) || inside(angle(corner.neighbors.tail));
        let head_is_inside = overlap(&head) || inside(angle(corner.neighbors.head));

        if tail_is_inside == head_is_inside {
            return None;
        }

        if head_is_inside {
            Some(Event::Entry)
        } else {
            Some(Event::Exit)
        }
    }

    fn side(&self, point: &Self::Vertex) -> Option<Side> {
        let determinant = Determinant::from([self.from, self.to, point]).into_inner();
        if determinant > T::zero() {
            return Some(Side::Left);
        }

        if determinant < T::zero() {
            return Some(Side::Right);
        }

        None
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float,
{
    /// Being zero the determinant of self and the other, returns the single common [`Point`]
    /// between them, if any.
    fn collinear_common_points(
        &self,
        other: &Segment<'_, T>,
    ) -> Option<MaybePair<Point<T>>> {
        let project_on_x = (self.to.x - self.from.x).abs() > (self.to.y - self.from.y).abs();
        let project = |point: &Point<T>| -> T {
            if project_on_x {
                point.x
            } else {
                point.y
            }
        };

        let self_from = project(self.from);
        let self_to = project(self.to);
        let other_from = project(other.from);
        let other_to = project(other.to);

        let first = T::max(self_from.min(self_to), other_from.min(other_to));
        let second = T::min(self_from.max(self_to), other_from.max(other_to));

        let unproject = |scalar: T| {
            // parametric function u along self
            let u = (scalar - project(self.from)) / (project(self.to) - project(self.from));
            (T::zero()..=T::one())
                .contains(&u)
                .then(|| *self.from + (*self.to - *self.from) * u)
        };

        if second < first {
            return Default::default();
        }

        if first == second {
            return unproject(first).map(MaybePair::Single);
        }

        match (unproject(first), unproject(second)) {
            (Some(first), Some(second)) => Some(MaybePair::Pair([first, second])),
            (Some(point), _) | (_, Some(point)) => Some(MaybePair::Single(point)),
            _ => Default::default(),
        }
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float,
{
    /// Returns the [`Determinant`] of the matrix representing the direction vectors of this and the
    /// other segment
    fn determinant(&self, other: &Self) -> Determinant<T> {
        Determinant::new(self, other)
    }
}

impl<T> Segment<'_, T>
where
    T: Signed + Float + FloatConst,
{
    /// Returns the distance between the two endpoints of the segment.
    fn length(&self) -> T {
        self.from.distance(self.to)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cartesian::{Point, Segment}, Corner, Neighbors, Intersection, Role, Edge, Event, MaybePair
    };

    #[test]
    fn intersection() {
        struct Test<'a> {
            name: &'a str,
            segment: Segment<'a, f64>,
            other: Segment<'a, f64>,
            want: Option<MaybePair<Point<f64>>>,
        }

        vec![
            Test {
                name: "non-crossing segments",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[8., 8.].into(),
                },
                other: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 0.].into(),
                },
                want: None,
            },
            Test {
                name: "perpendicular with no common endpoint",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 0.].into(),
                },
                want: Some(MaybePair::Single([2., 2.].into())),
            },
            Test {
                name: "perpendicular with endpoint in line",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 0.].into(),
                },
                other: Segment {
                    from: &[2., 2.].into(),
                    to: &[2., 0.].into(),
                },
                want: Some(MaybePair::Single([2., 0.].into())),
            },
            Test {
                name: "perpendicular segments starting at the same point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[-4., 4.].into(),
                },
                want: Some(MaybePair::Single([0., 0.].into())),
            },
            Test {
                name: "perpendicular segments ending at the same point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 8.].into(),
                    to: &[4., 4.].into(),
                },
                want: Some(MaybePair::Single([4., 4.].into())),
            },
            Test {
                name: "none-collinear parallel segments",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 4.].into(),
                    to: &[4., 8.].into(),
                },
                want: None,
            },
            Test {
                name: "collinear segments starting at the same point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[-4., -4.].into(),
                },
                want: Some(MaybePair::Single([0., 0.].into())),
            },
            Test {
                name: "collinear segments ending at the same point",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[0., 0.].into(),
                },
                other: Segment {
                    from: &[-4., -4.].into(),
                    to: &[0., 0.].into(),
                },
                want: Some(MaybePair::Single([0., 0.].into())),
            },
            Test {
                name: "collinear segments with no common point",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[-4., -4.].into(),
                    to: &[-2., -2.].into(),
                },
                want: None,
            },
            Test {
                name: "coincident segments when other is shorter",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[2., 2.].into(),
                },
                want: Some(MaybePair::Pair([[0., 0.].into(), [2., 2.].into()])),
            },
            Test {
                name: "coincident segments when other is larger",
                segment: Segment {
                    from: &[4., 4.].into(),
                    to: &[8., 8.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[8., 8.].into(),
                },
                want: Some(MaybePair::Pair([[4., 4.].into(), [8., 8.].into()])),
            },
            Test {
                name: "coincident segments when segment contains other",
                segment: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                other: Segment {
                    from: &[1., 1.].into(),
                    to: &[3., 3.].into(),
                },
                want: Some(MaybePair::Pair([[1., 1.].into(), [3., 3.].into()])),
            },
            Test {
                name: "coincident segments when other constains segment",
                segment: Segment {
                    from: &[1., 1.].into(),
                    to: &[3., 3.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[4., 4.].into(),
                },
                want: Some(MaybePair::Pair([[1., 1.].into(), [3., 3.].into()])),
            },
            Test {
                name: "coincident when none is fully contained",
                segment: Segment {
                    from: &[-1., 0.].into(),
                    to: &[1., 0.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[2., 0.].into(),
                },
                want: Some(MaybePair::Pair([[0., 0.].into(), [1., 0.].into()])),
            },
            Test {
                name: "coincident at oposite direction when none is fully contained",
                segment: Segment {
                    from: &[1., 0.].into(),
                    to: &[-1., 0.].into(),
                },
                other: Segment {
                    from: &[0., 0.].into(),
                    to: &[2., 0.].into(),
                },
                want: Some(MaybePair::Pair([[0., 0.].into(), [1., 0.].into()])),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.segment.intersection(&test.other, &Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }

    #[test]
    fn event() {
        struct Test<'a> {
            name: &'a str,
            corner: Corner<'a, Point<f64>>,
            want: Option<Event>,
        }

        vec![
            Test {
                name: "entering at edge",
                corner: Corner {
                    vertex: &[1., 1.].into(),
                    neighbors: Neighbors {
                        tail: &[0., 1.].into(),
                        head: &[2., 1.].into(),
                    },
                    role: Role::Subject,
                    intersection: Some(Intersection {
                        event: None,
                        neighbors: Neighbors {
                            tail: &[1., 2.].into(),
                            head: &[1., 0.].into(),
                        },
                    })
                },
                want: Some(Event::Entry),
            },
            Test {
                name: "exiting at edge",
                corner: Corner {
                    vertex: &[1., 1.].into(),
                    neighbors: Neighbors {
                        tail: &[0., 1.].into(),
                        head: &[2., 1.].into(),
                    },
                    role: Role::Subject,
                    intersection: Some(Intersection {
                        event: None,
                        neighbors: Neighbors {
                            tail: &[1., 0.].into(),
                            head: &[1., 2.].into(),
                        },
                    })
                },
                want: Some(Event::Exit),
            },
            Test {
                name: "entering at corner",
                corner: Corner {
                    vertex: &[0., 1.].into(),
                    neighbors: Neighbors {
                        tail: &[1., 2.].into(),
                        head: &[1., 0.].into(),
                    },
                    role: Role::Subject,
                    intersection: Some(Intersection { 
                        event: None, 
                        neighbors: Neighbors {
                            tail: &[1., 1.].into(),
                            head: &[0., 0.].into(),
                        },
                    })
                },
                want: Some(Event::Entry),
            },
            // Test {
            //     name: "exiting at corner",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 0.].into(),
            //         head: &[1., 2.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[1., 1.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     want: Some(Event::Exit),
            // },
            // Test {
            //     name: "touching edge from the inside",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 0.].into(),
            //         head: &[1., 2.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[0., 2.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     want: None,
            // },
            // Test {
            //     name: "touching edge from the outside",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 0.].into(),
            //         head: &[1., 2.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[0., 0.].into(),
            //         head: &[0., 2.].into(),
            //     },
            //     want: None,
            // },
            // Test {
            //     name: "joining edge from the inside",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 0.].into(),
            //         head: &[1., 1.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[1., 1.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     want: None,
            // },
            // Test {
            //     name: "joining edge from the outside",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 2.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[1., 1.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     want: Some(Event::Entry),
            // },
            // Test {
            //     name: "always on the edge",
            //     intersection: [0., 1.].into(),
            //     subject: Neighbors {
            //         tail: &[1., 1.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     other: Neighbors {
            //         tail: &[1., 1.].into(),
            //         head: &[0., 0.].into(),
            //     },
            //     want: None,
            // },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Default::default();
            let got = Segment::event(
                test.corner,
                &tolerance,
            );

            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

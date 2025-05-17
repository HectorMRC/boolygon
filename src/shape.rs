use std::{fmt::Debug, marker::PhantomData};

use num_traits::{Float, Signed};

use crate::{
    clipper::{Clipper, Operands, Operator, Role, Vertex},
    point::Point,
    polygon::Polygon,
};

/// A combination of disjoint [`Polygon`]s.
#[derive(Debug, Clone)]
pub struct Shape<T> {
    /// The list of non-crossing [`Polygon`]s.
    pub(crate) polygons: Vec<Polygon<T>>,
}

impl<T, P> From<T> for Shape<P>
where
    T: Into<Polygon<P>>,
    P: Signed + Float,
{
    fn from(value: T) -> Self {
        let polygon = value.into();

        Self {
            polygons: vec![if polygon.is_clockwise() {
                polygon.reversed()
            } else {
                polygon
            }],
        }
    }
}

impl<T> PartialEq for Shape<T>
where
    T: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        if self.polygons.len() != other.polygons.len() {
            return false;
        }

        self.polygons
            .iter()
            .all(|a| other.polygons.iter().any(|b| a.eq(b)))
    }
}

impl<T> Shape<T>
where
    T: PartialOrd + Signed + Float + Debug,
{
    /// Returns the union of self and rhs.
    pub fn or(self, rhs: Self) -> Self {
        struct OrOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for OrOperator<T>
        where
            T: Signed + Float,
        {
            fn is_output<'a>(ops: Operands<'a, T>, vertex: &'a Vertex<T>) -> bool {
                match vertex.role {
                    Role::Subject => !ops.clip.contains(&vertex.point),
                    Role::Clip => !ops.subject.contains(&vertex.point),
                    Role::Intersection => true,
                }
            }
        }

        Clipper::default()
            .with_operator::<OrOperator<T>>()
            .with_subject(self)
            .with_clip(rhs)
            .execute()
            .expect("union should always return a shape")
    }

    /// Returns the difference of rhs on self or [`None`] if no polygon remains.
    pub fn not(self, rhs: Self) -> Option<Self> {
        struct NotOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for NotOperator<T>
        where
            T: Signed + Float,
        {
            fn is_output<'a>(ops: Operands<'a, T>, vertex: &'a Vertex<T>) -> bool {
                match vertex.role {
                    Role::Subject => !ops.clip.contains(&vertex.point),
                    Role::Clip => ops.subject.contains(&vertex.point),
                    Role::Intersection => true,
                }
            }
        }

        Clipper::default()
            .with_operator::<NotOperator<T>>()
            .with_clip(rhs.inverted_winding())
            .with_subject(self)
            .execute()
    }
}

impl<T> Shape<T>
where
    T: Signed + Float,
{
    /// Returns the amount of times self winds around the given [`Point`].
    fn winding(&self, point: &Point<T>) -> isize {
        self.polygons
            .iter()
            .map(|polygon| polygon.winding(point))
            .sum()
    }

    /// Returns true if, and only if, self contains the given [`Point`].
    pub fn contains(&self, point: &Point<T>) -> bool {
        self.winding(point) != 0
    }

    /// Returns  a new shape with the inverted winding.
    fn inverted_winding(self) -> Self {
        Self {
            polygons: self.polygons.into_iter().map(Polygon::reversed).collect(),
        }
    }
}

impl<T> Shape<T> {
    /// Returns the amount of vertices in the shape.
    pub(crate) fn total_vertices(&self) -> usize {
        self.polygons
            .iter()
            .map(|polygon| polygon.vertices.len())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::shape::Shape;

    #[test]
    fn shape_union() {
        struct Test {
            name: &'static str,
            subject: Shape<f64>,
            clip: Shape<f64>,
            want: Shape<f64>,
        }

        vec![
            Test {
                name: "overlapping squares",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                want: vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 2.],
                    [6., 2.],
                    [6., 6.],
                    [2., 6.],
                    [2., 4.],
                    [0., 4.],
                ]
                .into(),
            },
            Test {
                name: "non-overlapping squares",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[6., 6.], [10., 6.], [10., 10.], [6., 10.]].into(),
                want: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[6., 6.], [10., 6.], [10., 10.], [6., 10.]].into(),
                    ],
                },
            },
            Test {
                name: "enclosing squares",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]].into(),
                want: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
            },
            Test {
                name: "subject enclosing clip",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                want: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                },
                clip: vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                want: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
            },
            Test {
                name: "subject with hole excluding clip",
                subject: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: vec![[1.5, 1.5], [2.5, 1.5], [2.5, 2.5], [1.5, 2.5]].into(),
                want: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                        vec![[1.5, 1.5], [2.5, 1.5], [2.5, 2.5], [1.5, 2.5]].into(),
                    ],
                },
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.or(test.clip);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }

    #[test]
    fn shape_difference() {
        struct Test {
            name: &'static str,
            subject: Shape<f64>,
            clip: Shape<f64>,
            want: Option<Shape<f64>>,
        }

        vec![
            Test {
                name: "overlapping squares",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                want: Some(vec![[0., 0.], [4., 0.], [4., 2.], [2., 2.], [2., 4.], [0., 4.]].into()),
            },
            Test {
                name: "subject enclosing clip",
                subject: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                clip: vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                want: Some(Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "clip enclosing subject",
                subject: vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                clip: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                want: None,
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                },
                clip: vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                want: Some(Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    polygons: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                want: Some(
                    vec![
                        [0., 0.],
                        [4., 0.],
                        [4., 2.],
                        [3., 2.],
                        [3., 1.],
                        [1., 1.],
                        [1., 3.],
                        [2., 3.],
                        [2., 4.],
                        [0., 4.],
                    ]
                    .into(),
                ),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.not(test.clip);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

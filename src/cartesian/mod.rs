mod determinant;
mod point;
mod polygon;
mod segment;

pub use self::point::Point;
pub use self::polygon::Polygon;
pub use self::segment::Segment;

#[cfg(test)]
mod tests {
    use crate::{cartesian::Polygon, Shape};

    #[test]
    fn union() {
        struct Test {
            name: &'static str,
            subject: Shape<Polygon<f64>>,
            clip: Shape<Polygon<f64>>,
            want: Option<Shape<Polygon<f64>>>,
        }

        vec![
            // Test {
            //     name: "same geometry",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            // },
            Test {
                name: "horizontally aligned squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 0.], [8., 0.], [8., 4.], [4., 4.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [8., 0.],
                    [8., 4.],
                    [4., 4.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "horizontal partial overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 4.], [6., 4.], [6., 8.], [2., 8.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 4.],
                    [6., 4.],
                    [6., 8.],
                    [2., 8.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            // Test {
            //     name: "horizontal overlapping squares",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[2., 0.], [6., 0.], [6., 4.], [2., 4.]]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [2., 0.],
            //         [4., 0.],
            //         [6., 0.],
            //         [6., 4.],
            //         [4., 4.],
            //         [2., 4.],
            //         [0., 4.],
            //     ])),
            // },
            Test {
                name: "diagonal overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 2.],
                    [6., 2.],
                    [6., 6.],
                    [2., 6.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "squares sharing a single vertex",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 4.], [8., 4.], [8., 8.], [4., 8.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[4., 4.], [8., 4.], [8., 8.], [4., 8.]].into(),
                    ],
                }),
            },
            Test {
                name: "squares sharing multiple vertices",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 4.], [3., 5.], [0., 4.], [1., 3.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 4.],
                    [3., 5.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "non-overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[6., 6.], [10., 6.], [10., 10.], [6., 10.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[6., 6.], [10., 6.], [10., 10.], [6., 10.]].into(),
                    ],
                }),
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                },
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            },
            Test {
                name: "subject with hole excluding clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[1.5, 1.5], [2.5, 1.5], [2.5, 2.5], [1.5, 2.5]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                        vec![[1.5, 1.5], [2.5, 1.5], [2.5, 2.5], [1.5, 2.5]].into(),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![
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
                        vec![[3., 2.], [3., 1.], [1., 1.], [1., 3.], [2., 3.], [2., 2.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip with hole",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                        vec![[3., 5.], [5., 5.], [5., 3.], [3., 3.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        vec![
                            [4., 2.],
                            [6., 2.],
                            [6., 6.],
                            [2., 6.],
                            [2., 4.],
                            [0., 4.],
                            [0., 0.],
                            [4., 0.],
                        ]
                        .into(),
                        vec![[4., 3.], [4., 4.], [3., 4.], [3., 5.], [5., 5.], [5., 3.]].into(),
                        vec![[3., 2.], [3., 1.], [1., 1.], [1., 3.], [2., 3.], [2., 2.]].into(),
                    ],
                }),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.or(test.clip, Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }

    #[test]
    fn difference() {
        struct Test {
            name: &'static str,
            subject: Shape<Polygon<f64>>,
            clip: Shape<Polygon<f64>>,
            want: Option<Shape<Polygon<f64>>>,
        }

        vec![
            Test {
                name: "same geometry",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                want: None,
            },
            Test {
                name: "horizontally aligned squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 0.], [8., 0.], [8., 4.], [4., 4.]]),
                want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            },
            Test {
                name: "horizontal partial overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 4.], [6., 4.], [6., 8.], [2., 8.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 4.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            // Test {
            //     name: "horizontal overlapping squares",
            //     subject: Shape::new(vec!([0., 0.], [4., 0.], [4., 4.], [0., 4.])),
            //     clip: Shape::new(vec!([2., 0.], [6., 0.], [6., 4.], [2., 4.])),
            //     want: Some(Shape::new(vec!(
            //         [0., 0.],
            //         [2., 0.],
            //         [2., 4.],
            //         [0., 4.]
            //     ))),
            // },
            Test {
                name: "diagonal overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 2.],
                    [2., 2.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "squares sharing a single vertex",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 4.], [8., 4.], [8., 8.], [4., 8.]]),
                want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            },
            Test {
                name: "squares sharing multiple vertices",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 4.], [3., 5.], [0., 4.], [1., 3.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [4., 0.],
                    [4., 4.],
                    [1., 3.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                want: None,
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                },
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 1.], [1., 3.], [3., 3.], [3., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape::new(vec![
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
                ])),
            },
            Test {
                name: "subject with hole intersecting clip with hole",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                        vec![[3., 5.], [5., 5.], [5., 3.], [3., 3.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
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
                        vec![[3., 3.], [4., 3.], [4., 4.], [3., 4.]].into(),
                    ],
                }),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.not(test.clip, Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }

    #[test]
    fn intersection() {
        struct Test {
            name: &'static str,
            subject: Shape<Polygon<f64>>,
            clip: Shape<Polygon<f64>>,
            want: Option<Shape<Polygon<f64>>>,
        }

        vec![
            // Test {
            //     name: "same geometry",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            // },
            Test {
                name: "horizontally aligned squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 0.], [8., 0.], [8., 4.], [4., 4.]]),
                want: Some(Shape::new(vec![[4., 0.], [4., 4.]])),
            },
            Test {
                name: "horizontal partial overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 4.], [6., 4.], [6., 8.], [2., 8.]]),
                want: Some(Shape::new(vec![[4., 4.], [2., 4.]])),
            },
            // Test {
            //     name: "horizontal overlapping squares",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[2., 0.], [6., 0.], [6., 4.], [2., 4.]]),
            //     want: Some(Shape::new(vec![[2., 0.], [4., 0.], [4., 4.], [2., 4.]])),
            // },
            Test {
                name: "diagonal overlapping squares",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape::new(vec![[2., 2.], [4., 2.], [4., 4.], [2., 4.]])),
            },
            // Test {
            //     name: "squares sharing a single vertex",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[4., 4.], [8., 4.], [8., 8.], [4., 8.]]),
            //     want: None,
            // },
            Test {
                name: "squares sharing multiple vertices",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[4., 4.], [3., 5.], [0., 4.], [1., 3.]]),
                want: Some(Shape::new(vec![[0., 4.], [1., 3.], [4., 4.]])),
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]])),
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
                want: Some(Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]])),
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                },
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape {
                    boundaries: vec![
                        vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]].into(),
                        vec![[1.5, 2.5], [2.5, 2.5], [2.5, 1.5], [1.5, 1.5]].into(),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]]),
                want: Some(Shape::new(vec![
                    [3., 2.],
                    [4., 2.],
                    [4., 4.],
                    [2., 4.],
                    [2., 3.],
                    [3., 3.],
                ])),
            },
            Test {
                name: "subject with hole intersecting clip with hole",
                subject: Shape {
                    boundaries: vec![
                        vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
                        vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                        vec![[3., 5.], [5., 5.], [5., 3.], [3., 3.]].into(),
                    ],
                },
                want: Some(Shape::new(vec![
                    [2., 4.],
                    [2., 3.],
                    [3., 3.],
                    [3., 2.],
                    [4., 2.],
                    [4., 3.],
                    [3., 3.],
                    [3., 4.],
                ])),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.and(test.clip, Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

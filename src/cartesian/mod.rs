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
            Test {
                name: "disjoint solid shapes",
                subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
                clip: Shape::new(vec![[2., 0.], [3., 0.], [3., 1.], [2., 1.]]),
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]].into(),
                        // Counter-clockwise boundary.
                        vec![[2., 0.], [3., 0.], [3., 1.], [2., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "solid subject partially overlaping solid clip",
                subject: Shape::new(vec![[0., 0.], [2., 0.], [2., 2.], [0., 2.]]),
                clip: Shape::new(vec![[1., 1.], [3., 1.], [3., 3.], [1., 3.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [2., 0.],
                    [2., 1.],
                    [3., 1.],
                    [3., 3.],
                    [1., 3.],
                    [1., 2.],
                    [0., 2.],
                ])),
            },
            Test {
                name: "enclosing-subject hole partially overlaping solid clip",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 3.], [3., 3.], [3., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[2., 2.], [4., 2.], [4., 4.], [2., 4.]]),
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 3.], [2., 3.], [2., 2.], [3., 2.], [3., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "solid subject partially overlaping enclosing-clip hole",
                subject: Shape::new(vec![[2., 2.], [4., 2.], [4., 4.], [2., 4.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 3.], [3., 3.], [3., 1.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 3.], [2., 3.], [2., 2.], [3., 2.], [3., 1.]].into(),
                    ],
                }),
            },
            Test {
                name: "enclosing-subject hole partially overlaping clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 4.], [4., 4.], [4., 2.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [6., 1.], [6., 6.], [1., 6.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 5.], [5., 5.], [5., 3.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject hole partially overlaping enclosing-clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [6., 1.], [6., 6.], [1., 6.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 5.], [5., 5.], [5., 3.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 4.], [4., 4.], [4., 2.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                }),
            },
            // Test {
            //     name: "overlaping solid shapes",
            //     subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
            //     clip: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]])),
            // },
            // Test {
            //     name: "solid subject partially overlaping solid clip with coincident edges",
            //     subject: Shape::new(vec![[0., 0.], [2., 0.], [2., 2.], [0., 2.]]),
            //     clip: Shape::new(vec![[1., 0.], [3., 0.], [3., 2.], [1., 2.]]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [1., 0.],
            //         [2., 0.],
            //         [3., 0.],
            //         [3., 2.],
            //         [2., 2.],
            //         [1., 2.],
            //         [0., 2.],
            //     ])),
            // },
            // Test {
            //     name: "enclosing-subject hole overlaping solid clip with coincident edges",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [3., 2.], [3., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape::new(vec![[2., 1.], [4., 1.], [4., 2.], [2., 2.]]),
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "solid subject overlaping enclosing-clip hole with coincident edges",
            //     subject: Shape::new(vec![[2., 1.], [4., 1.], [4., 2.], [2., 2.]]),
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [3., 2.], [3., 1.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "enclosing-subject hole overlaping clip hole with coincident edges",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [7., 0.], [7., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [4., 3.], [4., 2.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [6., 1.], [6., 4.], [1., 4.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 2.], [3., 3.], [5., 3.], [5., 2.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [7., 0.], [7., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [4., 3.], [4., 2.], [3., 2.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "subject hole overlaping enclosing-clip hole with coincident edges",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [6., 1.], [6., 4.], [1., 4.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 2.], [3., 3.], [5., 3.], [5., 2.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [7., 0.], [7., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [4., 3.], [4., 2.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [7., 0.], [7., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [4., 3.], [4., 2.], [3., 2.]].into(),
            //         ],
            //     }),
            // },
            Test {
                name: "solid subject traversing solid clip",
                subject: Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]]),
                clip: Shape::new(vec![[2., 1.], [4., 1.], [4., 2.], [2., 2.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [3., 0.],
                    [3., 1.],
                    [4., 1.],
                    [4., 2.],
                    [3., 2.],
                    [3., 3.],
                    [0., 3.],
                ])),
            },
            Test {
                name: "enclosing-subject hole overlaping clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [4., 1.], [4., 4.], [1., 4.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject hole overlaping enclosing-clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [4., 1.], [4., 4.], [1., 4.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                }),
            },
            Test {
                name: "solid subject overlaping clip hole",
                subject: Shape::new(vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
                    ],
                },
                want: Some(Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]])),
            },
            // Test {
            //     name: "subject hole overlaping solid clip",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape::new(vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]])),
            // },
            Test {
                name: "solid subject enclosing solid clip",
                subject: Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]]),
                clip: Shape::new(vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]]),
                want: Some(Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]])),
            },
            Test {
                name: "solid subject enclosing clip hole",
                subject: Shape::new(vec![[1., 1.], [4., 1.], [4., 4.], [1., 4.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                want: Some(Shape::new(vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]])),
            },
            Test {
                name: "subject hole enclosing solid clip",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 4.], [4., 4.], [4., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]]),
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 4.], [4., 4.], [4., 1.]].into(),
                        // Counter-clockwise boundary.
                        vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject hole enclosing clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 5.], [5., 5.], [5., 2.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [6., 1.], [6., 6.], [1., 6.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                }),
            },
            Test {
                name: "solid subject inside solid clip",
                subject: Shape::new(vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]]),
                clip: Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]]),
                want: Some(Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]])),
            },
            Test {
                name: "subject hole inside solid clip",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                clip: Shape::new(vec![[1., 1.], [4., 1.], [4., 4.], [1., 4.]]),
                want: Some(Shape::new(vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]])),
            },
            Test {
                name: "solid subject inside clip hole",
                subject: Shape::new(vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 4.], [4., 4.], [4., 1.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 4.], [4., 4.], [4., 1.]].into(),
                        // Counter-clockwise boundary.
                        vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]].into(),
                    ],
                }),
            },
            Test {
                name: "subject hole inside clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [6., 1.], [6., 6.], [1., 6.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 5.], [5., 5.], [5., 2.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
                    ],
                }),
            },
            // Test {
            //     name: "solid subject sharing vertex with solid clip",
            //     subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
            //     clip: Shape::new(vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]]),
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]].into(),
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "enclosing subject sharing vertex with solid clip",
            //     subject: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[1., 1.], [2., 2.], [1., 3.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]])),
            // },
            Test {
                name: "solid subject sharing vertex with enclosing clip",
                subject: Shape::new(vec![[1., 1.], [2., 2.], [1., 3.]]),
                clip: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [2., 0.],
                    [2., 2.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "solid subject sharing vertex with clip hole",
                subject: Shape::new(vec![[3., 3.], [4., 3.], [4., 4.], [3., 4.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
                    ],
                }),
            },
            // Test {
            //     name: "subject hole sharing vertex with solid clip",
            //     subject: Shape{
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
            //         ]
            //     },
            //     clip: Shape::new(vec![[3., 3.], [4., 3.], [4., 4.], [3., 4.]]),
            //     want: Some(Shape{
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
            //         ]
            //     }),
            // },
            Test {
                name: "enclosing-subject hole sharing vertex with clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [4., 3.], [4., 4.], [3., 4.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                        // Clockwise boundary.
                        vec![[4., 4.], [4., 5.], [5., 5.], [5., 4.]].into(),
                    ],
                },
                want: Some(Shape::new(vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]])),
            },
            Test {
                name: "subject hole sharing vertex with enclosing-clip hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
                        // Clockwise boundary.
                        vec![[4., 4.], [4., 5.], [5., 5.], [5., 4.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [4., 3.], [4., 4.], [3., 4.]].into(),
                    ],
                },
                want: Some(Shape::new(vec![[0., 0.], [7., 0.], [7., 7.], [0., 7.]])),
            },
            Test {
                name: "enclosing-subject hole sharing vertex with clip hole inside subject hole",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [6., 0.], [6., 8.], [0., 8.]].into(),
                        // Clockwise boundary.
                        vec![[2., 2.], [2., 6.], [4., 6.], [4., 2.]].into(),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[1., 1.], [5., 1.], [5., 7.], [1., 7.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 5.], [4., 4.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [6., 0.], [6., 8.], [0., 8.]].into(),
                        // Clockwise boundary.
                        vec![[3., 3.], [3., 5.], [4., 4.]].into(),
                    ],
                }),
            },
            // Test {
            //     name: "subject hole inside clip hole sharing vertex with enclosing-clip hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [5., 1.], [5., 7.], [1., 7.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 5.], [4., 4.]].into(),
            //         ]
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 8.], [0., 8.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 6.], [4., 6.], [4., 2.]].into(),
            //         ]
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 8.], [0., 8.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 5.], [4., 4.]].into(),
            //         ]
            //     })
            // },
            Test {
                name: "solid subject sharing edge with exterior solid clip",
                subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
                clip: Shape::new(vec![[1., 0.], [2., 0.], [2., 1.], [1., 1.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [1., 0.],
                    [2., 0.],
                    [2., 1.],
                    [1., 1.],
                    [0., 1.],
                ])),
            },
            // Test {
            //     name: "enclosing solid subject sharing edge with solid clip",
            //     subject: Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]]),
            //     clip: Shape::new(vec![[2., 1.], [3., 1.], [3., 2.], [2., 2.]]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [3., 0.],
            //         [3., 1.],
            //         [3., 2.],
            //         [3., 3.],
            //         [0., 3.],
            //     ])),
            // },
            // Test {
            //     name: "solid subject sharing edge with enclosing solid clip",
            //     subject: Shape::new(vec![[2., 1.], [3., 1.], [3., 2.], [2., 2.]]),
            //     clip: Shape::new(vec![[0., 0.], [3., 0.], [3., 3.], [0., 3.]]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [3., 0.],
            //         [3., 1.],
            //         [3., 2.],
            //         [3., 3.],
            //         [0., 3.],
            //     ])),
            // },
            // Test {
            //     name: "enclosing-subject hole sharing edge with solid clip",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape::new(vec![[2., 1.], [3., 1.], [3., 2.], [2., 2.]]),
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "solid subject sharing edge with enclosing-clip hole",
            //     subject: Shape::new(vec![[2., 1.], [3., 1.], [3., 2.], [2., 2.]]),
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 3.], [0., 3.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 2.], [2., 2.], [2., 1.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "subject hole sharing edge with solid clip",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 4.], [3., 4.], [3., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape::new(vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]]),
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [4., 0.], [4., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![
            //                 [1., 1.],
            //                 [1., 4.],
            //                 [3., 4.],
            //                 [3., 3.],
            //                 [2., 3.],
            //                 [2., 2.],
            //                 [3., 2.],
            //                 [3., 1.],
            //             ]
            //             .into(),
            //         ],
            //     }),
            // },
            Test {
                name: "solid subject sharing edge with clip hole",
                subject: Shape::new(vec![[2., 2.], [3., 2.], [3., 3.], [2., 3.]]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [4., 0.], [4., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 4.], [3., 4.], [3., 1.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [4., 0.], [4., 5.], [0., 5.]].into(),
                        // Clockwise boundary.
                        vec![
                            [1., 1.],
                            [1., 4.],
                            [3., 4.],
                            [3., 3.],
                            [2., 3.],
                            [2., 2.],
                            [3., 2.],
                            [3., 1.],
                        ]
                        .into(),
                    ],
                }),
            },
            // Test {
            //     name: "enclosing-subject hole sharing edge with clip hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
            //         ]
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [5., 1.], [5., 4.], [1., 4.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 2.], [3., 3.], [4., 3.], [4., 2.]].into(),
            //         ]
            //     },
            //     want: Some(Shape::new(vec![[0., 0.], [6., 0.], [6., 5.], [0., 5.]]))
            // },
            // Test {
            //     name: "subject hole sharing edge with enclosing-clip hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [5., 1.], [5., 4.], [1., 4.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 2.], [3., 3.], [4., 3.], [4., 2.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 5.], [0., 5.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 3.], [3., 3.], [3., 2.]].into(),
            //         ],
            //     },
            //     want: Some(Shape::new(vec![[0., 0.], [6., 0.], [6., 5.], [0., 5.]])),
            // },
            // Test {
            //     name: "enclosing-subject hole sharing edge with clip hole inside subject hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 7.], [0., 7.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 5.], [4., 5.], [4., 2.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [5., 1.], [5., 6.], [1., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 7.], [0., 7.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "subject hole inside clip hole sharing edge with enclosing-clip hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[1., 1.], [5., 1.], [5., 6.], [1., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 7.], [0., 7.]].into(),
            //             // Clockwise boundary.
            //             vec![[2., 2.], [2., 5.], [4., 5.], [4., 2.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [6., 0.], [6., 7.], [0., 7.]].into(),
            //             // Clockwise boundary.
            //             vec![[3., 3.], [3., 4.], [4., 4.], [4., 3.]].into(),
            //         ],
            //     }),
            // },
            Test {
                name: "subject sharing edge with entry clip",
                subject: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
                clip: Shape::new(vec![
                    [1., 1.],
                    [3., 1.],
                    [3., 3.],
                    [2., 3.],
                    [2., 2.],
                    [1., 2.],
                ]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [2., 0.],
                    [2., 1.],
                    [3., 1.],
                    [3., 3.],
                    [2., 3.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            Test {
                name: "entry subject sharing edge with clip",
                subject: Shape::new(vec![
                    [1., 1.],
                    [3., 1.],
                    [3., 3.],
                    [2., 3.],
                    [2., 2.],
                    [1., 2.],
                ]),
                clip: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
                want: Some(Shape::new(vec![
                    [0., 0.],
                    [2., 0.],
                    [2., 1.],
                    [3., 1.],
                    [3., 3.],
                    [2., 3.],
                    [2., 4.],
                    [0., 4.],
                ])),
            },
            // Test {
            //     name: "subject sharing edge with exit clip",
            //     subject: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![
            //         [1., 1.],
            //         [2., 1.],
            //         [2., 2.],
            //         [3., 2.],
            //         [3., 3.],
            //         [1., 3.],
            //     ]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [2., 0.],
            //         [2., 1.],
            //         [2., 2.],
            //         [3., 2.],
            //         [3., 3.],
            //         [2., 3.],
            //         [2., 4.],
            //         [0., 4.],
            //     ])),
            // },
            // Test {
            //     name: "exit subject sharing edge with clip",
            //     subject: Shape::new(vec![
            //         [1., 1.],
            //         [2., 1.],
            //         [2., 2.],
            //         [3., 2.],
            //         [3., 3.],
            //         [1., 3.],
            //     ]),
            //     clip: Shape::new(vec![[0., 0.], [2., 0.], [2., 4.], [0., 4.]]),
            //     want: Some(Shape::new(vec![
            //         [0., 0.],
            //         [2., 0.],
            //         [2., 1.],
            //         [2., 2.],
            //         [3., 2.],
            //         [3., 3.],
            //         [2., 3.],
            //         [2., 4.],
            //         [0., 4.],
            //     ])),
            // },
            // Test {
            //     name: "subject hole sharing edge with exit clip",
            //     subject: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 5.], [3., 5.], [3., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape::new(vec![
            //         [2., 2.],
            //         [4., 2.],
            //         [4., 4.],
            //         [3., 4.],
            //         [3., 3.],
            //         [2., 3.],
            //     ]),
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![
            //                 [1., 1.],
            //                 [1., 5.],
            //                 [3., 5.],
            //                 [3., 4.],
            //                 [3., 3.],
            //                 [2., 3.],
            //                 [2., 2.],
            //                 [3., 2.],
            //                 [3., 1.],
            //             ]
            //             .into(),
            //         ],
            //     }),
            // },
            // Test {
            //     name: "exit subject sharing edge with clip hole",
            //     subject: Shape::new(vec![
            //         [2., 2.],
            //         [4., 2.],
            //         [4., 4.],
            //         [3., 4.],
            //         [3., 3.],
            //         [2., 3.],
            //     ]),
            //     clip: Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![[1., 1.], [1., 5.], [3., 5.], [3., 1.]].into(),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             // Counter-clockwise boundary.
            //             vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
            //             // Clockwise boundary.
            //             vec![
            //                 [1., 1.],
            //                 [1., 5.],
            //                 [3., 5.],
            //                 [3., 4.],
            //                 [3., 3.],
            //                 [2., 3.],
            //                 [2., 2.],
            //                 [3., 2.],
            //                 [3., 1.],
            //             ]
            //             .into(),
            //         ],
            //     }),
            // },
            Test {
                name: "subject hole sharing edge with entry clip",
                subject: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 5.], [3., 5.], [3., 1.]].into(),
                    ],
                },
                clip: Shape::new(vec![
                    [2., 2.],
                    [3., 2.],
                    [3., 3.],
                    [4., 3.],
                    [4., 4.],
                    [2., 4.],
                ]),
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
                        // Clockwise boundary.
                        vec![
                            [1., 1.],
                            [1., 5.],
                            [3., 5.],
                            [3., 4.],
                            [2., 4.],
                            [2., 2.],
                            [3., 2.],
                            [3., 1.],
                        ]
                        .into(),
                    ],
                }),
            },
            Test {
                name: "entry subject sharing edge with clip hole",
                subject: Shape::new(vec![
                    [2., 2.],
                    [3., 2.],
                    [3., 3.],
                    [4., 3.],
                    [4., 4.],
                    [2., 4.],
                ]),
                clip: Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
                        // Clockwise boundary.
                        vec![[1., 1.], [1., 5.], [3., 5.], [3., 1.]].into(),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        // Counter-clockwise boundary.
                        vec![[0., 0.], [5., 0.], [5., 6.], [0., 6.]].into(),
                        // Clockwise boundary.
                        vec![
                            [1., 1.],
                            [1., 5.],
                            [3., 5.],
                            [3., 4.],
                            [2., 4.],
                            [2., 2.],
                            [3., 2.],
                            [3., 1.],
                        ]
                        .into(),
                    ],
                }),
            },
        ]
        .into_iter()
        // .filter(|test| test.name == "enclosing-subject hole partially overlaping clip hole")
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
            // Test {
            //     name: "same geometry",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     want: None,
            // },
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
            // Test {
            //     name: "squares sharing a single vertex",
            //     subject: Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]]),
            //     clip: Shape::new(vec![[4., 4.], [8., 4.], [8., 8.], [4., 8.]]),
            //     want: Some(Shape::new(vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]])),
            // },
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
            // Test {
            //     name: "subject with hole intersecting clip with hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
            //             vec![[1., 3.], [3., 3.], [3., 1.], [1., 1.]].into(),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
            //             vec![[3., 5.], [5., 5.], [5., 3.], [3., 3.]].into(),
            //         ],
            //     },
            //     want: Some(Shape::new(vec![
            //         [2., 4.],
            //         [2., 3.],
            //         [3., 3.],
            //         [3., 2.],
            //         [4., 2.],
            //         [4., 3.],
            //         [3., 3.],
            //         [3., 4.],
            //     ])),
            // },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.and(test.clip, Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

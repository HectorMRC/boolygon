mod arc;
mod point;
mod polygon;

pub use self::arc::Arc;
pub use self::point::{Azimuth, Inclination, Point};
pub use self::polygon::{spherical_polygon, Polygon};

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

    use crate::{spherical::Polygon, spherical_polygon, Shape, Tolerance};

    #[test]
    fn union() {
        struct Test {
            name: &'static str,
            subject: Shape<Polygon<f64>>,
            clip: Shape<Polygon<f64>>,
            want: Shape<Polygon<f64>>,
        }

        vec![
            // Test {
            //     name: "overlapping triangles",
            //     subject: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 0.],
            //         [FRAC_PI_2, FRAC_PI_2];
            //         [PI, 0.]
            //     )),
            //     clip: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
            //         [FRAC_PI_2, FRAC_PI_4];
            //         [PI, 0.]
            //     )),
            //     want: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
            //         [FRAC_PI_2, 0.],
            //         [FRAC_PI_2, FRAC_PI_4],
            //         [FRAC_PI_2, FRAC_PI_2];
            //         [FRAC_PI_2, 3. * FRAC_PI_2]
            //     )),
            // },
            Test {
                name: "triangles sharing vertex",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                    ],
                },
            },
            Test {
                name: "non-overlapping triangles",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [PI, 0.];
                    [0., 0.]
                )),
                want: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2];
                            [PI, PI]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_2, 3. * FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [PI, 0.];
                            [0., 0.]
                        ),
                    ],
                },
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                want: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
            },
            Test {
                name: "subject intersecting clip",
                subject: Shape::new(spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [FRAC_PI_2, PI];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                )),
                want: Shape {
                    boundaries: vec![spherical_polygon!(
                        [FRAC_PI_2, 0.],
                        [PI, 0.],
                        [FRAC_PI_2, PI],
                        [FRAC_PI_2, 3. * FRAC_PI_2];
                        [PI - FRAC_PI_8, 3. * FRAC_PI_2]
                    )],
                },
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_4 + FRAC_PI_8, 0.],
                    [FRAC_PI_4 + FRAC_PI_8, FRAC_PI_2],
                    [FRAC_PI_4 + FRAC_PI_8, PI],
                    [FRAC_PI_4 + FRAC_PI_8, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Shape::new(spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
            },
            Test {
                name: "subject with hole excluding clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, 0.],
                    [FRAC_PI_8, FRAC_PI_2],
                    [FRAC_PI_8, PI],
                    [FRAC_PI_8, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_8, 0.],
                            [FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_8, PI],
                            [FRAC_PI_8, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                    ],
                },
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [FRAC_PI_2, PI];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                )),
                want: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [PI, 0.],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI - FRAC_PI_8, 3. * FRAC_PI_2]
                        ),
                        spherical_polygon!(
                           [FRAC_PI_4, 0.],
                           [FRAC_PI_4, 3. * FRAC_PI_2],
                           [FRAC_PI_4, PI],
                           [0., 0.];
                           [PI, 0.]
                        ),
                    ],
                },
            },
            Test {
                name: "subject with hole intersecting clip with hole",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, 0.],
                            [PI, 0.],
                            [FRAC_PI_2, PI];
                            [FRAC_PI_2, 3. * FRAC_PI_2]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_2, PI - FRAC_PI_4],
                            [PI - FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_2, FRAC_PI_4],
                            [FRAC_PI_8, FRAC_PI_2];
                            [FRAC_PI_2, 3. * FRAC_PI_2]
                        ),
                    ],
                },
                want: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [PI, 0.],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI - FRAC_PI_8, 3. * FRAC_PI_2]
                        ),
                        spherical_polygon!(
                           [FRAC_PI_4, 0.],
                           [FRAC_PI_4, 3. * FRAC_PI_2],
                           [FRAC_PI_4, PI],
                           [0., 0.];
                           [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_2, PI - FRAC_PI_4],
                            [PI - FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_2, FRAC_PI_4],
                            [FRAC_PI_2, FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [0.6532635808587185, 1.9634954084936205],
                            [FRAC_PI_4, FRAC_PI_2],
                            [0.6532635808587185, 1.1780972450961726],
                            [0.39269908169872414, 1.5707963267948966];
                            [PI, 0.]
                        ),
                    ],
                },
            },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.or(test.clip, tolerance);
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
            //     name: "overlapping triangles",
            //     subject: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 0.],
            //         [FRAC_PI_2, FRAC_PI_2];
            //         [PI, PI]
            //     )),
            //     clip: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
            //         [FRAC_PI_2, FRAC_PI_4];
            //         [PI, PI]
            //     )),
            //     want: Some(Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, FRAC_PI_4],
            //         [FRAC_PI_2, FRAC_PI_2];
            //         [FRAC_PI_2, 3. * FRAC_PI_2]
            //     ))),
            // },
            Test {
                name: "non-overlapping triangles",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [PI, 0.],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [0., 0.]
                )),
                want: Some(Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                ))),
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                want: None,
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                want: Some(Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2];
                            [PI, PI]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_8, FRAC_PI_4],
                            [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8],
                            [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8];
                            [PI, PI]
                        ),
                    ],
                }),
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_4 + FRAC_PI_8, 0.],
                    [FRAC_PI_4 + FRAC_PI_8, FRAC_PI_2],
                    [FRAC_PI_4 + FRAC_PI_8, PI],
                    [FRAC_PI_4 + FRAC_PI_8, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Some(Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4 + FRAC_PI_8, 3. * FRAC_PI_2],
                            [FRAC_PI_4 + FRAC_PI_8, PI],
                            [FRAC_PI_4 + FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_4 + FRAC_PI_8, 0.];
                            [PI, 0.]
                        ),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [FRAC_PI_2, PI];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                )),
                want: Some(Shape::new(spherical_polygon!(
                    [FRAC_PI_4, 0.],
                    [FRAC_PI_4, 3. * FRAC_PI_2],
                    [FRAC_PI_4, PI],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2],
                    [FRAC_PI_2, 0.];
                    [PI, 0.]
                ))),
            },
            // Test {
            //     name: "subject with hole intersecting clip with hole",
            //     subject: Shape {
            //         boundaries: vec![
            //             spherical_polygon!(
            //                 [FRAC_PI_2, 0.],
            //                 [FRAC_PI_2, FRAC_PI_2],
            //                 [FRAC_PI_2, PI],
            //                 [FRAC_PI_2, 3. * FRAC_PI_2];
            //                 [PI, 0.]
            //             ),
            //             spherical_polygon!(
            //                 [FRAC_PI_4, 3. * FRAC_PI_2],
            //                 [FRAC_PI_4, PI],
            //                 [FRAC_PI_4, FRAC_PI_2],
            //                 [FRAC_PI_4, 0.];
            //                 [PI, 0.]
            //             ),
            //         ],
            //     },
            //     clip: Shape {
            //         boundaries: vec![
            //             spherical_polygon!(
            //                 [0., 0.],
            //                 [FRAC_PI_2, 0.],
            //                 [PI, 0.],
            //                 [FRAC_PI_2, PI];
            //                 [FRAC_PI_2, 3. * FRAC_PI_2]
            //             ),
            //             spherical_polygon!(
            //                 [FRAC_PI_2, PI - FRAC_PI_4],
            //                 [PI - FRAC_PI_8, FRAC_PI_2],
            //                 [FRAC_PI_2, FRAC_PI_4],
            //                 [FRAC_PI_8, FRAC_PI_2];
            //                 [FRAC_PI_2, 3. * FRAC_PI_2]
            //             ),
            //         ],
            //     },
            //     want: Some(Shape {
            //         boundaries: vec![
            //             spherical_polygon!(
            //                 [FRAC_PI_4, 0.],
            //                 [FRAC_PI_4, 3. * FRAC_PI_2],
            //                 [FRAC_PI_4, PI],
            //                 [FRAC_PI_2, PI],
            //                 [FRAC_PI_2, 3. * FRAC_PI_2],
            //                 [FRAC_PI_2, 0.];
            //                 [PI, 0.]
            //             ),
            //             spherical_polygon!(
            //                 [FRAC_PI_2, FRAC_PI_4],
            //                 [FRAC_PI_2, FRAC_PI_2],
            //                 [FRAC_PI_2, PI - FRAC_PI_4],
            //                 [0.6532635808587185, 1.9634954084936205],
            //                 [FRAC_PI_4, FRAC_PI_2],
            //                 [0.6532635808587185, 1.1780972450961726];
            //                 [PI, 0.]
            //             ),
            //         ],
            //     }),
            // },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.not(test.clip, tolerance);
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
            //     name: "overlapping triangles",
            //     subject: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 0.],
            //         [FRAC_PI_2, FRAC_PI_2];
            //         [PI, PI]
            //     )),
            //     clip: Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 3. * FRAC_PI_2 + FRAC_PI_4],
            //         [FRAC_PI_2, FRAC_PI_4];
            //         [PI, PI]
            //     )),
            //     want: Some(Shape::new(spherical_polygon!(
            //         [0., 0.],
            //         [FRAC_PI_2, 0.],
            //         [FRAC_PI_2, FRAC_PI_4];
            //         [FRAC_PI_2, 3. * FRAC_PI_2]
            //     ))),
            // },
            Test {
                name: "non-overlapping triangles",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [PI, 0.],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_2, 3. * FRAC_PI_2];
                    [0., 0.]
                )),
                want: None,
            },
            Test {
                name: "clip enclosing subject",
                subject: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Some(Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                ))),
            },
            Test {
                name: "subject enclosing clip",
                subject: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2];
                    [PI, 0.]
                )),
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                )),
                want: Some(Shape::new(spherical_polygon!(
                    [FRAC_PI_8, FRAC_PI_4],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_8],
                    [FRAC_PI_2 - FRAC_PI_8, FRAC_PI_2 - FRAC_PI_8];
                    [PI, 0.]
                ))),
            },
            Test {
                name: "subject with hole enclosing clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [FRAC_PI_4 + FRAC_PI_8, 0.],
                    [FRAC_PI_4 + FRAC_PI_8, FRAC_PI_2],
                    [FRAC_PI_4 + FRAC_PI_8, PI],
                    [FRAC_PI_4 + FRAC_PI_8, 3. * FRAC_PI_2];
                    [PI, 0.]
                )),
                want: Some(Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_4 + FRAC_PI_8, 0.],
                            [FRAC_PI_4 + FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_4 + FRAC_PI_8, PI],
                            [FRAC_PI_4 + FRAC_PI_8, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                }),
            },
            Test {
                name: "subject with hole intersecting clip",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape::new(spherical_polygon!(
                    [0., 0.],
                    [FRAC_PI_2, 0.],
                    [PI, 0.],
                    [FRAC_PI_2, PI];
                    [FRAC_PI_2, 3. * FRAC_PI_2]
                )),
                want: Some(Shape::new(spherical_polygon!(
                    [FRAC_PI_2, 0.],
                    [FRAC_PI_2, FRAC_PI_2],
                    [FRAC_PI_2, PI],
                    [FRAC_PI_4, PI],
                    [FRAC_PI_4, FRAC_PI_2],
                    [FRAC_PI_4, 0.];
                    [PI, 0.]
                ))),
            },
            Test {
                name: "subject with hole intersecting clip with hole",
                subject: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_2],
                            [FRAC_PI_2, PI],
                            [FRAC_PI_2, 3. * FRAC_PI_2];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_4, 3. * FRAC_PI_2],
                            [FRAC_PI_4, PI],
                            [FRAC_PI_4, FRAC_PI_2],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                    ],
                },
                clip: Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [0., 0.],
                            [FRAC_PI_2, 0.],
                            [PI, 0.],
                            [FRAC_PI_2, PI];
                            [FRAC_PI_2, 3. * FRAC_PI_2]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_2, PI - FRAC_PI_4],
                            [PI - FRAC_PI_8, FRAC_PI_2],
                            [FRAC_PI_2, FRAC_PI_4],
                            [FRAC_PI_8, FRAC_PI_2];
                            [FRAC_PI_2, 3. * FRAC_PI_2]
                        ),
                    ],
                },
                want: Some(Shape {
                    boundaries: vec![
                        spherical_polygon!(
                            [FRAC_PI_2, 0.],
                            [FRAC_PI_2, FRAC_PI_4],
                            [0.6532635808587185, 1.1780972450961726],
                            [FRAC_PI_4, 0.];
                            [PI, 0.]
                        ),
                        spherical_polygon!(
                            [FRAC_PI_2, PI],
                            [FRAC_PI_4, PI],
                            [0.6532635808587185, 1.9634954084936205],
                            [FRAC_PI_2, PI - FRAC_PI_4];
                            [PI, 0.]
                        ),
                    ],
                }),
            },
        ]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.and(test.clip, tolerance);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

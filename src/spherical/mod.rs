mod arc;
mod point;
mod polygon;

pub use self::arc::Arc;
pub use self::point::{Azimuth, Inclination, Point};
pub use self::polygon::{Polygon, spherical_polygon};

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, PI};

    use crate::{Shape, Tolerance, spherical::Polygon, spherical_polygon};

    #[test]
    fn union() {
        struct Test {
            name: &'static str,
            subject: Shape<Polygon<f64>>,
            clip: Shape<Polygon<f64>>,
            want: Option<Shape<Polygon<f64>>>,
        }

        vec![Test {
            name: "disjoint shapes",
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
            want: Some(Shape {
                boundaries: vec![
                    // Counter-clockwise boundary.
                    spherical_polygon!(
                        [0., 0.],
                        [FRAC_PI_2, 0.],
                        [FRAC_PI_2, FRAC_PI_2];
                        [PI, 0.]
                    ),
                    // Counter-clockwise boundary.
                    spherical_polygon!(
                        [PI, 0.],
                        [FRAC_PI_2, PI],
                        [FRAC_PI_2, 3. * FRAC_PI_2];
                        [0., 0.]
                    ),
                ],
            }),
        }]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.or(&test.clip, tolerance);
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

        vec![Test {
            name: "disjoint shapes",
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
        }]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.not(&test.clip, tolerance);
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

        vec![Test {
            name: "disjoint shapes",
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
        }]
        .into_iter()
        .for_each(|test| {
            let tolerance = Tolerance {
                relative: 1e-09.into(),
                absolute: 0.0.into(),
            };

            let got = test.subject.and(&test.clip, tolerance);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

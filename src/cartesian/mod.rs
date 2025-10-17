mod determinant;
mod point;
mod polygon;
mod segment;

pub use self::point::Point;
pub use self::polygon::Polygon;
pub use self::segment::Segment;

#[cfg(test)]
mod tests {
    use crate::{Shape, cartesian::Polygon};

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
        }]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.or(&test.clip, Default::default());
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
            subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
            clip: Shape::new(vec![[2., 0.], [3., 0.], [3., 1.], [2., 1.]]),
            want: Some(Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]])),
        }]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.not(&test.clip, Default::default());
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
            subject: Shape::new(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]),
            clip: Shape::new(vec![[2., 0.], [3., 0.], [3., 1.], [2., 1.]]),
            want: None,
        }]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.and(&test.clip, Default::default());
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

use std::fmt::Debug;

use num_traits::{Float, Signed};

use crate::{clipper::Clipper, point::Point, polygon::Polygon};

/// A combination of disjoint [`Polygon`]s.
#[derive(Debug, Clone)]
pub struct Shape<T> {
    /// The list of non-crossing [`Polygon`]s.
    pub(crate) polygons: Vec<Polygon<T>>,
}

impl<T, P> From<T> for Shape<P>
where
    T: Into<Polygon<P>>,
{
    fn from(value: T) -> Self {
        Self {
            polygons: vec![value.into()],
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
        Clipper::new(())
            .with_subject(self)
            .with_clip(rhs)
            .execute()
            .expect("union should always return a shape")
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
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.subject.or(test.clip);
            assert_eq!(got, test.want, "{}", test.name);
        });
    }
}

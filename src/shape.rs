use num_traits::{Float, Signed};

use crate::{point::Point, polygon::Polygon};

/// A combination of disjoint [`Polygon`]s.
#[derive(Debug)]
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

    /// Returns the difference of rhs on self.
    pub fn not(self, rhs: Self) -> Self {
        todo!()
    }

    /// Returns the union of self and rhs.
    pub fn or(self, rhs: Self) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::shape::Shape;

    #[test]
    fn shape_union() {
        struct Test {
            name: &'static str,
            left: Shape<f64>,
            right: Shape<f64>,
            want: Shape<f64>,
        }

        vec![Test {
            name: "overlapping squares",
            left: vec![[0., 0.], [4., 0.], [4., 4.], [0., 4.]].into(),
            right: vec![[2., 2.], [6., 2.], [6., 6.], [2., 6.]].into(),
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
        }]
        .into_iter()
        .for_each(|test| {
            let got = test.left.or(test.right);
            assert_eq!(
                got, test.want,
                "{} got = {got:?}, want = {:?}",
                test.name, test.want
            );
        });
    }
}

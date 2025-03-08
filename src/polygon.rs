use std::ops::{Add, Mul, Sub};

use crate::point::Point;

/// Represents the straight line between two vertices of a [`Polygon`].
pub struct Segment<'a, T = f64> {
    /// The first point in the segment.
    pub from: &'a Point<T>,
    /// The last point in the segment.
    pub to: &'a Point<T>,
}

impl<'a, T> From<(&'a Point<T>, &'a Point<T>)> for Segment<'a, T> {
    fn from((from, to): (&'a Point<T>, &'a Point<T>)) -> Self {
        Self { from, to }
    }
}

impl<T> Segment<'_, T>
where
    T: Copy + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    /// Returns the scalar cross product of the triangle resulting from self and the given
    /// [`Point`].
    pub fn cross(&self, point: &Point<T>) -> T {
        (self.to.x - self.from.x) * (point.y - self.from.y)
            - (point.x - self.from.x) * (self.to.y - self.from.y)
    }
}

/// Represents a closed shape in the plain.
pub struct Polygon<T = f64> {
    /// The ordered list of vertices describing the polygon.  
    pub vertices: Vec<Point<T>>,
}

impl<T, P> From<Vec<P>> for Polygon<T>
where
    P: Into<Point<T>>,
{
    fn from(vertices: Vec<P>) -> Self {
        Self {
            vertices: vertices.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T> Polygon<T>
where
    T: PartialOrd
        + Copy
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + num_traits::Signed,
{
    /// Returns the amount of times self winds around the given [`Point`].
    pub fn winding(&self, point: &Point<T>) -> isize {
        self.segments().fold(0, |wn, segment| {
            if segment.from.y <= point.y
                && segment.to.y > point.y
                && segment.cross(point).is_positive()
            {
                wn + 1
            } else if segment.from.y > point.y
                && segment.to.y <= point.y
                && segment.cross(point).is_negative()
            {
                wn - 1
            } else {
                wn
            }
        })
    }

    /// Returns true if, and only if, self contains the given point.
    pub fn contains(&self, point: &Point<T>) -> bool {
        self.winding(point) != 0
    }
}

impl<T> Polygon<T> {
    /// Returns an ordered iterator over all the vertices of the polygon.
    ///
    /// By definition, a polygon is a closed shape, hence the latest point of the iterator will be
    /// equal to the very first one.
    pub fn vertices(&self) -> impl Iterator<Item = &Point<T>> {
        self.vertices.iter().chain(self.vertices.first())
    }

    /// Returns an ordered iterator over all the segments that make up the polygon.
    pub fn segments(&self) -> impl Iterator<Item = Segment<'_, T>> {
        self.vertices()
            .zip(self.vertices().skip(1))
            .map(Segment::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{point::Point, polygon::Polygon};

    #[test]
    fn winding_number() {
        struct Test {
            name: &'static str,
            polygon: Polygon,
            point: Point,
            want: isize,
        }

        vec![
            Test {
                name: "Center of a counterclockwise polygon.",
                polygon: vec![[4., 0.], [4., 4.], [0., 4.], [0., 0.]].into(),
                point: [2., 2.].into(),
                want: 1,
            },
            Test {
                name: "Center of a clockwise polygon.",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [2., 2.].into(),
                want: -1,
            },
            Test {
                name: "On the left of the polygon.",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [-2., -2.].into(),
                want: 0,
            },
            Test {
                name: "On the right of the polygon.",
                polygon: vec![[0., 0.], [0., 4.], [4., 4.], [4., 0.]].into(),
                point: [6., 6.].into(),
                want: 0,
            },
            Test {
                name: "Inside self-crossing polygon.",
                polygon: vec![
                    [8., 0.],
                    [8., 6.],
                    [2., 6.],
                    [2., 4.],
                    [6., 4.],
                    [6., 2.],
                    [4., 2.],
                    [4., 8.],
                    [0., 8.],
                    [0., 0.],
                ]
                .into(),
                point: [3., 5.].into(),
                want: 2,
            },
            Test {
                name: "Outside self-crossing polygon.",
                polygon: vec![
                    [8., 0.],
                    [8., 6.],
                    [2., 6.],
                    [2., 4.],
                    [6., 4.],
                    [6., 2.],
                    [4., 2.],
                    [4., 8.],
                    [0., 8.],
                    [0., 0.],
                ]
                .into(),
                point: [5., 3.].into(),
                want: 0,
            },
        ]
        .into_iter()
        .for_each(|test| {
            let got = test.polygon.winding(&test.point);
            assert_eq!(
                got, test.want,
                "{} got winding number = {got}, want = {}",
                test.name, test.want
            );
        });
    }
}

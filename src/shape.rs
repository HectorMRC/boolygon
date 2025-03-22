use crate::polygon::Polygon;

/// A [`Polygon`] containing other polygons inside.
///
/// It is guaranteed that none of the children intersect with each other nor intersect with the
/// polygon's boundaries.
struct PolygonCluster<T> {
    polygon: Polygon<T>,
    children: Vec<PolygonCluster<T>>,
}

impl<T> From<Polygon<T>> for PolygonCluster<T> {
    fn from(polygon: Polygon<T>) -> Self {
        Self {
            polygon,
            children: Default::default(),
        }
    }
}

/// Represents a combination of non-crossing [`Polygon`]s.
#[derive(Default)]
struct Shape<T> {
    /// The hierarchically ordered list of [`Polygon`]s involved in the shape.
    clusters: Vec<PolygonCluster<T>>,
}

impl<T> From<Polygon<T>> for Shape<T> {
    fn from(polygon: Polygon<T>) -> Self {
        Self {
            clusters: vec![polygon.into()],
        }
    }
}

impl<T> Shape<T> {
    /// Returns the intersection of self and rhs.
    fn and(self, rhs: Self) -> Self {
        todo!()
    }

    /// Returns the difference of rhs on self.
    fn not(self, rhs: Self) -> Self {
        todo!()
    }

    /// Returns the union of self and rhs.
    fn or(self, rhs: Self) -> Self {
        todo!()
    }
}

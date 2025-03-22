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
    /// Returns the intersection of self and the given [`Polygon`].
    fn and(self, polygon: Polygon<T>) -> Self {
        todo!()
    }

    /// Returns the difference of the given [`Polygon`] on self.
    fn not(self, polygon: Polygon<T>) -> Self {
        todo!()
    }

    /// Returns the union of self and the given [`Polygon`].
    fn or(self, polygon: Polygon<T>) -> Self {
        todo!()
    }
}

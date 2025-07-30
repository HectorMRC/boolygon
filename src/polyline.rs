/// An ordered list of vertices describing an open shape.
pub struct Polyline<T>(Vec<T>);

impl<T> From<Vec<T>> for Polyline<T> {
    fn from(vertices: Vec<T>) -> Self {
        Self(vertices)
    }
}

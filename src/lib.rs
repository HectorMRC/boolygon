mod clipper;
mod graph;
mod node;
mod shape;
mod tolerance;

#[cfg(feature = "cartesian")]
pub mod cartesian;
#[cfg(feature = "spherical")]
pub mod spherical;

pub use self::clipper::Operands;
pub use self::shape::Shape;
pub use self::tolerance::{IsClose, Positive, Tolerance};

/// A vertex from a [`Geometry`].
pub trait Vertex {
    /// The scalar type in this vertex's space.
    type Scalar;

    /// Returns the distance between this vertex and rhs.
    fn distance(&self, rhs: &Self) -> Self::Scalar;
}

/// An edge delimited by two vertices in a [`Geometry`].
pub trait Edge<'a> {
    /// The endpoint type of the edge.
    type Vertex: Vertex + IsClose;

    /// Returns an edge from the given endpoints.
    fn new(from: &'a Self::Vertex, to: &'a Self::Vertex) -> Self;

    /// Returns the middle point of this edge.
    fn midpoint(&self) -> Self::Vertex;

    /// Returns true if, and only if, the given point exists in this edge.
    fn contains(
        &self,
        point: &Self::Vertex,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> bool;

    /// Returns the intersection between this edge and rhs, if any.
    fn intersection(
        &self,
        rhs: &Self,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> Option<Self::Vertex>;
}

/// A [`Geometry`] whose orientation is defined by the right-hand rule.
pub trait RightHanded {
    /// Returns true if, and only if, this geometry is oriented clockwise.
    fn is_clockwise(&self) -> bool;
}

/// A geometry in an arbitrary space.
pub trait Geometry: Sized + RightHanded {
    /// The type of the vertices this geometry is made of.
    type Vertex: Vertex + IsClose;

    /// The type of the edges this geometry is made of.
    type Edge<'a>: Edge<'a, Vertex = Self::Vertex>
    where
        Self: 'a;

    /// Tries to construct a geometry from the given raw data.
    fn from_raw(
        operands: Operands<Self>,
        vertices: Vec<Self::Vertex>,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> Option<Self>;

    /// Returns the total amount of vertices in the geometry.
    fn total_vertices(&self) -> usize;

    /// Returns an ordered iterator over all the segmentss of this geometry.
    fn edges(&self) -> impl Iterator<Item = Self::Edge<'_>>;

    /// Returns this geometry with the reversed orientation.
    fn reversed(self) -> Self;

    /// Returns the amount of times this geometry winds around the given vertex.
    fn winding(
        &self,
        vertex: &Self::Vertex,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> isize;
}

mod clipper;
mod graph;
mod pair;
mod shape;
mod tolerance;

#[cfg(feature = "cartesian")]
pub mod cartesian;
#[cfg(feature = "spherical")]
pub mod spherical;

pub use self::pair::MaybePair;
pub use self::shape::Shape;
pub use self::tolerance::{IsClose, Positive, Tolerance};
pub use self::clipper::Context;

/// A vertex from a [`Geometry`].
pub trait Vertex: IsClose {
    /// The scalar type in this vertex's space.
    type Scalar;

    /// Returns the distance between this vertex and the other.
    fn distance(&self, other: &Self) -> Self::Scalar;
}

pub enum Side {
    Left,
    Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The boundary is entering into the other.
    Entry,
    /// The boundary is exiting from the other.
    Exit,
}

/// The local information of a vertex.
pub struct Neighbors<'a, T> {
    /// The vertex before.
    pub tail: &'a T,
    /// The vertex after.
    pub head: &'a T,
}

pub struct Intersection<'a, T> {
    pub event: Option<Event>,
    pub neighbors: Neighbors<'a, T>,
}

/// The role of an arbitrary entity during a clipping operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Subject,
    Clip,
}

impl Role {
    /// Returns true if, and only if, this is [`Role::Subject`].
    pub(crate) fn is_subject(&self) -> bool {
        matches!(self, Self::Subject)
    }
}

pub struct Corner<'a, T> {
    pub vertex: &'a T,
    pub neighbors: Neighbors<'a, T>,  
    pub role: Role,
    pub intersection: Option<Intersection<'a, T>>
}

/// An edge delimited by two vertices in a [`Geometry`].
pub trait Edge<'a>: Sized {
    /// The endpoint type of the edge.
    type Vertex: Vertex;

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

    /// Returns the intersection between this edge and the other, if any.
    fn intersection(
        &self,
        other: &Self,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> Option<MaybePair<Self::Vertex>>;

    /// Returns the [`Event`] of the given intersection vertex and local information.
    fn event(
        corner: Corner<'a, Self::Vertex>,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> Option<Event>;

    fn side(&self, point: &Self::Vertex) -> Option<Side>;
}

/// A geometry in an arbitrary space.
pub trait Geometry: Sized {
    /// The type of the vertices this geometry is made of.
    type Vertex: Vertex;

    /// The type of the edges this geometry is made of.
    type Edge<'a>: Edge<'a, Vertex = Self::Vertex>
    where
        Self: 'a;

    /// Tries to construct a geometry from the given raw data.
    fn from_raw(
        operands: Context<Self>,
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

    /// Returns true if, and only if, this geometry is oriented clockwise.
    fn is_clockwise(&self) -> bool;
}

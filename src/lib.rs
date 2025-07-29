mod clipper;
mod graph;
mod node;
mod tolerance;

#[cfg(feature = "cartesian")]
pub mod cartesian;
#[cfg(feature = "spherical")]
pub mod spherical;

pub use self::clipper::Operands;
pub use self::tolerance::{IsClose, Positive, Tolerance};

use std::{fmt::Debug, marker::PhantomData};

use self::{
    clipper::{Clipper, Operator},
    node::{Node, Role},
};

/// A vertex from a [`Geometry`].
pub trait Vertex {
    /// The scalar type in this vertex's space.
    type Scalar;

    /// Returns the distance between this vertex and rhs.
    fn distance(&self, rhs: &Self) -> Self::Scalar;
}

/// An edge delimited by two vertices in a [`Geometry`].
pub trait Edge<'a, T>
where
    T: Vertex + IsClose,
{
    /// Returns an edge from the given endpoints.
    fn new(from: &'a T, to: &'a T) -> Self;

    /// Returns the middle point of this edge.
    fn midpoint(&self) -> T;

    /// Returns true if, and only if, the given point exists in this edge.
    fn contains(&self, point: &T, tolerance: &T::Tolerance) -> bool;

    /// Returns the intersection between this edge and rhs, if any.
    fn intersection(&self, rhs: &Self, tolerance: &T::Tolerance) -> Option<T>;
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
    type Edge<'a>: Edge<'a, Self::Vertex>
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

    /// Returns this geometry with the reversed winding.
    fn reversed(self) -> Self;

    /// Returns the amount of times this geometry winds around the given vertex.
    fn winding(
        &self,
        vertex: &Self::Vertex,
        tolerance: &<Self::Vertex as IsClose>::Tolerance,
    ) -> isize;
}

/// A combination of disjoint boundaries.
#[derive(Debug, Clone)]
pub struct Shape<T> {
    /// The list of non-crossing boundaries.
    pub(crate) boundaries: Vec<T>,
}

impl<T> From<T> for Shape<T>
where
    T: Geometry,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> PartialEq for Shape<T>
where
    T: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        if self.boundaries.len() != other.boundaries.len() {
            return false;
        }

        self.boundaries
            .iter()
            .all(|a| other.boundaries.iter().any(|b| a.eq(b)))
    }
}

impl<T> Shape<T>
where
    T: Geometry + Clone + IntoIterator<Item = T::Vertex>,
    T::Vertex: Copy + PartialEq + PartialOrd,
    <T::Vertex as Vertex>::Scalar: Copy + PartialOrd,
{
    /// Returns the union of this shape and rhs.
    pub fn or(self, rhs: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Self {
        struct OrOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for OrOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                node: &'a Node<T>,
                tolerance: &<T::Vertex as IsClose>::Tolerance,
            ) -> bool {
                match node.role {
                    Role::Subject(_) => {
                        !ops.clip.contains(&node.vertex, tolerance)
                            || ops.clip.is_boundary(&node.vertex, tolerance)
                    }
                    Role::Clip(_) => {
                        !ops.subject.contains(&node.vertex, tolerance)
                            || ops.subject.is_boundary(&node.vertex, tolerance)
                    }
                }
            }
        }

        Clipper::default()
            .with_operator::<OrOperator<T>>()
            .with_tolerance(tolerance)
            .with_subject(self)
            .with_clip(rhs)
            .execute()
            .expect("union should always return a shape")
    }

    /// Returns the difference of rhs on this shape.
    pub fn not(self, rhs: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Option<Self> {
        struct NotOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for NotOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                node: &'a Node<T>,
                tolerance: &<T::Vertex as IsClose>::Tolerance,
            ) -> bool {
                match node.role {
                    Role::Subject(_) => {
                        !ops.clip.contains(&node.vertex, tolerance)
                            && !ops.clip.is_boundary(&node.vertex, tolerance)
                    }
                    Role::Clip(_) => {
                        ops.subject.contains(&node.vertex, tolerance)
                            && !ops.subject.is_boundary(&node.vertex, tolerance)
                    }
                }
            }
        }

        Clipper::default()
            .with_operator::<NotOperator<T>>()
            .with_tolerance(tolerance)
            .with_clip(rhs.inverted_winding())
            .with_subject(self)
            .execute()
    }

    /// Returns the intersection of this shape and rhs.
    pub fn and(self, rhs: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Option<Self> {
        struct AndOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for AndOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(
                ops: Operands<'a, T>,
                node: &'a Node<T>,
                tolerance: &<T::Vertex as IsClose>::Tolerance,
            ) -> bool {
                match node.role {
                    Role::Subject(_) => {
                        ops.clip.contains(&node.vertex, tolerance)
                            || ops.clip.is_boundary(&node.vertex, tolerance)
                    }
                    Role::Clip(_) => {
                        ops.subject.contains(&node.vertex, tolerance)
                            || ops.subject.is_boundary(&node.vertex, tolerance)
                    }
                }
            }
        }

        Clipper::default()
            .with_operator::<AndOperator<T>>()
            .with_tolerance(tolerance)
            .with_subject(self)
            .with_clip(rhs)
            .execute()
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, the given [`Vertex`] lies on the boundaries of this shape.
    pub(crate) fn is_boundary(
        &self,
        vertex: &T::Vertex,
        tolerance: &<T::Vertex as IsClose>::Tolerance,
    ) -> bool {
        self.boundaries
            .iter()
            .flat_map(|boundary| boundary.edges())
            .any(|segment| segment.contains(vertex, tolerance))
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    T::Vertex: Vertex,
{
    /// Returns the amount of times this shape winds around the given [`Vertex`].
    fn winding(&self, vertex: &T::Vertex, tolerance: &<T::Vertex as IsClose>::Tolerance) -> isize {
        self.boundaries
            .iter()
            .map(|boundary| boundary.winding(vertex, tolerance))
            .sum()
    }

    /// Returns true if, and only if, the given [`Vertex`] lies inside this shape.
    pub(crate) fn contains(
        &self,
        vertex: &T::Vertex,
        tolerance: &<T::Vertex as IsClose>::Tolerance,
    ) -> bool {
        self.winding(vertex, tolerance) != 0
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Creates a new shape from the given boundary.
    pub fn new(value: impl Into<T>) -> Self {
        let boundary = value.into();

        Self {
            boundaries: vec![if boundary.is_clockwise() {
                boundary.reversed()
            } else {
                boundary
            }],
        }
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Returns  a new shape with the inverted winding.
    fn inverted_winding(self) -> Self {
        Self {
            boundaries: self.boundaries.into_iter().map(T::reversed).collect(),
        }
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Returns the amount of vertices in this shape.
    pub(crate) fn total_vertices(&self) -> usize {
        self.boundaries
            .iter()
            .map(|boundary| boundary.total_vertices())
            .sum()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = T::Edge<'_>> {
        self.boundaries.iter().flat_map(|boundary| boundary.edges())
    }
}

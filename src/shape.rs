use std::{fmt::Debug, marker::PhantomData};

use crate::{
    clipper::{Clipper, Direction, Operator}, graph::{BoundaryRole, Intersection, Node}, Edge, Geometry, IsClose, Operands, Vertex
};

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
                match node.boundary {
                    BoundaryRole::Subject => {
                        !ops.clip.contains(&node.vertex, tolerance)
                            
                    }
                    BoundaryRole::Clip => {
                        !ops.subject.contains(&node.vertex, tolerance)
                            
                    }
                }
            }

            fn direction(node: &Node<T>) -> Direction {
                let Some(intersection) = node.intersection else {
                    return Direction::Forward;
                };

                match intersection {
                    Intersection::Entry => Direction::Backward,
                    Intersection::Exit => Direction::Forward,
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
                match node.boundary {
                    BoundaryRole::Subject => {
                        !ops.clip.contains(&node.vertex, tolerance)
                    }
                    BoundaryRole::Clip => {
                        ops.subject.contains(&node.vertex, tolerance)
                    }
                }
            }

            fn direction(node: &Node<T>) -> Direction {
                let Some(intersection) = node.intersection else {
                    return if node.boundary.is_subject() {
                        Direction::Forward
                    } else {
                        Direction::Backward
                    }
                };

                match (node.boundary, intersection) {
                    (BoundaryRole::Subject, Intersection::Entry) => Direction::Backward,
                    (BoundaryRole::Subject, Intersection::Exit) => Direction::Forward,
                    (BoundaryRole::Clip, Intersection::Entry) => Direction::Forward,
                    (BoundaryRole::Clip, Intersection::Exit) => Direction::Backward,
                }
            }
        }

        Clipper::default()
            .with_operator::<NotOperator<T>>()
            .with_tolerance(tolerance)
            .with_clip(rhs)
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
                match node.boundary {
                    BoundaryRole::Subject => {
                        ops.clip.contains(&node.vertex, tolerance)
                            
                    }
                    BoundaryRole::Clip => {
                        ops.subject.contains(&node.vertex, tolerance)
                            
                    }
                }
            }

            fn direction(node: &Node<T>) -> Direction {
                let Some(intersection) = node.intersection else {
                    return Direction::Forward;
                };

                match intersection {
                    Intersection::Entry => Direction::Forward,
                    Intersection::Exit => Direction::Backward,
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

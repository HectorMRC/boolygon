use std::fmt::Debug;

use crate::{
    clipper::{Clipper, Operator},
    graph::Graph,
    Element, Geometry, Shape,
};

/// Determines the role of a [`Vertex`] during the clipping process.
#[derive(Debug, Clone, Copy)]
pub(super) enum Role {
    /// The vertex belongs to the subject shape.
    Subject,
    /// The vertex belongs to the clip shape.
    Clip,
}

#[derive(Debug)]
pub(super) struct Vertex<T>
where
    T: Geometry,
{
    /// The location of the vertex.
    pub(super) point: T::Point,
    /// The role of the vertex.
    pub(super) role: Role,
    /// The index of the vertex following this one.
    pub(super) next: usize,
    /// The index of the vertex previous to this one.
    pub(super) previous: usize,
    /// Vertices from the oposite shape located at the same point.
    pub(super) siblings: Vec<usize>,
}

impl<T> Vertex<T>
where
    T: Geometry,
{
    pub(super) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

pub(super) struct VerticesIterator<'a, Op, T>
where
    T: Geometry,
{
    pub(super) clipper: &'a Clipper<<T::Point as Element>::Scalar, Op, Shape<T>, Shape<T>>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) next: Option<usize>,
    pub(super) init: usize,
}

impl<Op, T> Iterator for VerticesIterator<'_, Op, T>
where
    T: Geometry,
    T::Point: Copy + PartialEq,
    <T::Point as Element>::Scalar: Copy,
    Op: Operator<T>,
{
    type Item = Vertex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.unwrap_or(self.init);
        if self.graph.vertices[next]
            .as_ref()?
            .siblings
            .contains(&self.init)
        {
            // The polygon is already closed.
            return None;
        }

        let vertex = self.graph.vertices[next].take()?;
        self.next = self.clipper.select_path(self.graph, &vertex);

        if let Some(previous) = self
            .next
            .and_then(|next| self.graph.vertices[next].as_ref())
            .map(|next| next.previous)
        {
            self.graph.vertices[previous].take();
        }

        Some(vertex)
    }
}

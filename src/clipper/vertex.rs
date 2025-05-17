use std::fmt::Debug;

use num_traits::{Float, Signed};

use crate::{point::Point, shape::Shape};

use super::{graph::Graph, Clipper, Operator};

/// Determines the role of a [`Vertex`] during the clipping process.
#[derive(Debug, Clone, Copy)]
pub enum Role {
    /// The vertex belongs to the subject shape.
    Subject,
    /// The vertex belongs to the clip shape.
    Clip,
    /// The vertex is an intersection, therefore it belongs to both the subject and clip shapes.
    Intersection,
}

#[derive(Debug)]
pub struct Vertex<T> {
    /// The location of the vertex.
    pub point: Point<T>,
    /// The role of the vertex.
    pub role: Role,
    /// The index of the vertex following this one.
    pub(super) next: usize,
    /// The index of the vertex previous to this one.
    pub(super) previous: usize,
    /// Vertices from the oposite shape located at the same point.
    pub(super) siblings: Vec<usize>,
}

impl<T> Vertex<T> {
    pub(super) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

pub(super) struct VerticesIterator<'a, Op, T> {
    pub(super) clipper: &'a Clipper<Op, Shape<T>, Shape<T>>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) next: usize,
}

impl<'a, Op, T> Iterator for VerticesIterator<'a, Op, T>
where
    T: Signed + Float,
    Op: Operator<T>,
{
    type Item = Vertex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let vertex = self.graph.vertices[self.next].take()?;

        self.next = vertex
            .siblings
            .iter()
            .find_map(|&sibling| {
                if self.graph.vertices[sibling]
                    .as_ref()
                    .and_then(|vertex| self.graph.vertices[vertex.next].as_ref())
                    .is_some_and(|vertex| Op::is_output(self.clipper.into(), vertex))
                {
                    return self.graph.vertices[sibling]
                        .take()
                        .map(|vertex| vertex.next);
                }

                None
            })
            .unwrap_or(vertex.next);

        Some(vertex)
    }
}

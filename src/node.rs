use std::{collections::BTreeSet, fmt::Debug};

use crate::{
    clipper::{Clipper, Operator},
    graph::Graph,
    Geometry, IsClose, Shape,
};

/// Determines the role of a [`Node`] during the clipping process.
#[derive(Debug, Clone, Copy)]
pub(super) enum Role {
    /// The node belongs to the subject shape.
    Subject(usize),
    /// The node belongs to the clip shape.
    Clip(usize),
}

impl Role {
    pub(super) fn is_subject(&self) -> bool {
        matches!(self, Role::Subject(_))
    }
}

/// A vertex and its metadata inside a graph.
#[derive(Debug)]
pub(super) struct Node<T>
where
    T: Geometry,
{
    /// The vertex being represented by this node.
    pub(super) vertex: T::Vertex,
    /// The role of the node.
    pub(super) role: Role,
    /// The index of the node following this one.
    pub(super) next: usize,
    /// The index of the node previous to this one.
    pub(super) previous: usize,
    /// Vertices from the oposite shape located at the same point.
    pub(super) siblings: BTreeSet<usize>,
}

impl<T> Node<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, this node has siblings.
    pub(super) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

/// An iterator of [`Node`] that yields consecutive items from a [`Graph`] which vertex belongs to
/// the output boundary.
pub(super) struct NodeIterator<'a, Op, T>
where
    T: Geometry,
{
    pub(super) clipper: &'a Clipper<Op, Shape<T>, Shape<T>, <T::Vertex as IsClose>::Tolerance>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) next: Option<usize>,
}

impl<Op, T> Iterator for NodeIterator<'_, Op, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.graph.nodes[self.next?].take()?;
        self.next = self.clipper.select_path(self.graph, &node);
        Some(node)
    }
}

/// A wrapper iterator around [`NodeIterator`] that stops yielding vertices when the boundary forms
/// a closed shape.
pub(super) struct BoundaryCollector<'a, Op, T>
where
    T: Geometry,
{
    iterator: NodeIterator<'a, Op, T>,
    terminal: Vec<usize>,
    closed: bool,
}

impl<'a, Op, T> From<NodeIterator<'a, Op, T>> for BoundaryCollector<'a, Op, T>
where
    T: Geometry,
{
    fn from(iterator: NodeIterator<'a, Op, T>) -> Self {
        Self {
            iterator,
            terminal: Vec::new(),
            closed: false,
        }
    }
}

impl<Op, T> Iterator for &mut BoundaryCollector<'_, Op, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None;
        }

        let current = self.iterator.next?;
        let node = self.iterator.next()?;

        if self.terminal.is_empty() {
            self.terminal
                .extend(node.siblings.iter().copied().chain([current]));
        } else if let Some(next) = self.iterator.next {
            self.closed = self.terminal.contains(&next);
        } else {
            self.closed = self
                .iterator
                .graph
                .successors(&node)
                .any(|node| self.terminal.contains(&node));
        };

        Some(node)
    }
}

impl<Op, T> BoundaryCollector<'_, Op, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    /// Returns a vector of vertices if, and only if, the resulting boundary forms a closed shape.
    pub(super) fn collect(mut self) -> Option<Vec<T::Vertex>> {
        let vertices = self.map(|node| node.vertex).collect();
        self.closed.then_some(vertices)
    }
}

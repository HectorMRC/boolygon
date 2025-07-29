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
    pub(super) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }
}

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

        if let Some(previous) = self
            .next
            .and_then(|next| self.graph.nodes[next].as_ref())
            .map(|next| next.previous)
        {
            self.graph.nodes[previous].take();
        }

        Some(node)
    }
}

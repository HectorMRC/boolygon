use crate::{Geometry, graph::Node};

/// A direction to follow when traversing a boundary.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    /// Use the `next` field of the [`Node`].
    #[default]
    Forward,
    /// Use the `previous` field of the [`Node`].
    Backward,
}

impl Direction {
    /// Returns the index of the node following the given one.
    pub(crate) fn next<T>(&self, node: &Node<T>) -> usize
    where
        T: Geometry,
    {
        match self {
            Direction::Forward => node.next,
            Direction::Backward => node.previous,
        }
    }

    pub(crate) fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
    }

    pub(crate) fn reverse(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

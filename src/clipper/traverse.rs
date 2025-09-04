use crate::{graph::{Graph, Node}, Geometry, IsClose, Shape};

use super::{Clipper, Operator};

/// Yields all the nodes from a boundary.
pub(super) struct Traverse<'a, T, Op, Tol>
where
    T: Geometry,
{
    pub(super) clipper: &'a Clipper<Shape<T>, Shape<T>, Op, Tol>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) next: Option<usize>,
    pub(super) start: usize,
}

impl<'a, T, Op, Tol> Iterator for Traverse<'a, T, Op, Tol>
where
    T: Geometry,
    T::Vertex: Copy,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next
            && current == self.start
        {
            return None;
        }

        let current = self.next.unwrap_or(self.start);
        
        self.graph.vertices[current].visited = true;
        let vertex = &self.graph.vertices[current];
        self.next = Some(vertex.next);

        Some(*vertex)
    }
}

impl<T, Op, Tol> Traverse<'_, T, Op, Tol>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq + IsClose<Tolerance = Tol>,
    Op: Operator<T>,
{
    /// Returns the full path yielded by this iterator.
    pub(super) fn collect(self) -> Vec<T::Vertex> {
        let corner = self.graph.corner(self.start);
        let orientation = Op::direction(self.clipper.into(), corner);
        let mut boundary = self.map(|node| node.vertex).collect::<Vec<_>>();

        if !orientation.is_forward() {
            boundary.reverse();
        }

        boundary
    }
}


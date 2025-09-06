use crate::{graph::{Graph, Node}, Geometry, IsClose, MaybePair, Shape};

use super::{Clipper, Direction, Operator};

/// Yields each [`Node`] from the [`Graph`] within the path starting at the given position.
pub(super) struct Clip<'a, T, Op, Tol>
where
    T: Geometry,
{
    pub(super) clipper: &'a Clipper<Shape<T>, Shape<T>, Op, Tol>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) direction: Direction,
    pub(super) next: Option<usize>,
    pub(super) start: usize,
}

impl<T, Op, Tol> Iterator for Clip<'_, T, Op, Tol>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq + IsClose<Tolerance = Tol>,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.unwrap_or(self.start);
        let vertex = self.graph.take(current)?;

        let Some(intersection) = &vertex.intersection else {
           self.next = Some(self.direction.next(&vertex));
           return Some(vertex);
        };

        let Some(sibling) = self.graph.get(intersection.sibling) else {
            if intersection.event.is_some() {
                let corner = self.graph.corner(current);
                self.direction = Op::direction(self.clipper.into(), corner);
            }
            
            self.next = Some(self.direction.next(&vertex));
            return Some(vertex);
        };

        if sibling.intersection.as_ref().and_then(|intersection| intersection.event).is_some() {
            let corner = self.graph.corner(intersection.sibling);
            self.direction = Op::direction(self.clipper.into(), corner);
            self.next = Some(self.direction.next(&sibling));
            return Some(vertex)
        }

        // Handle degenerate cases.
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl<T, Op, Tol> Clip<'_, T, Op, Tol>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq + IsClose<Tolerance = Tol>,
    Op: Operator<T>,
{
    /// Returns the full boundary yielded by this iterator.
    pub(super) fn collect(mut self) -> Option<Vec<T::Vertex>> {
        let first = &self.graph.vertices[self.start];
        let terminal = match &first.intersection {
            Some(intersection) => MaybePair::Pair([self.start, intersection.sibling]),
            None => MaybePair::Single(self.start),
        };

        let (lower, _) = self.size_hint();
        let mut boundary = Vec::with_capacity(lower);
        while !self.next.is_some_and(|next| terminal.contains(&next))  {
            boundary.push(self.next()?.vertex);
        }

        if !self.direction.is_forward() {
            boundary.reverse();
        }

        Some(boundary)
    }
}
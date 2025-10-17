use crate::{
    Corner, Edge, Geometry, Intersection, IsClose, MaybePair, Neighbors, Shape,
    direction::Direction,
    graph::{Graph, Node},
};

use super::{Clipper, Operator};

/// Yields each [`Node`] from the [`Graph`] within the path starting at the given position.
pub(super) struct Clip<'a, T, Op, Tol>
where
    T: Geometry,
{
    pub(super) clipper: &'a Clipper<'a, Shape<T>, Shape<T>, Op, Tol>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) direction: Option<Direction>,
    pub(super) next: Option<usize>,
    pub(super) start: usize,
}

impl<T, Op, Tol> Iterator for Clip<'_, T, Op, Tol>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Edge<'a>,
    T::Vertex: Copy + PartialEq + IsClose<Tolerance = Tol>,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.unwrap_or(self.start);
        let node = self.graph.take(current)?;

        let Some(intersection) = &node.intersection else {
            self.next = self.direction.map(|direction| direction.next(&node));
            return Some(node);
        };

        let Some(sibling) = self.graph.get(intersection.sibling) else {
            self.direction =
                Op::direction(self.clipper.into(), self.graph.corner(current)).or(self.direction);

            self.next = self.direction.map(|direction| direction.next(&node));
            return Some(node);
        };

        if let Some(direction) =
            Op::direction(self.clipper.into(), self.graph.corner(intersection.sibling))
        {
            self.next = Some(direction.next(&sibling));
            self.direction = Some(direction);
            return Some(node);
        }

        // Handle degenerate cases.

        if !Op::is_output(self.clipper.into(), self.graph.corner(current)) {
            return None;
        }

        if let Some(direction) = Op::direction(self.clipper.into(), self.graph.corner(current))
            && self.direction.unwrap_or(direction) == direction
        {
            // Changing direction means moving along a path already traveled.
            // If the direction doesn't change at this vertex, it's safe to continue without switching of boundary.
            self.next = Some(direction.next(&sibling));
            self.direction = Some(direction);
            return Some(node);
        }

        self.direction = self.direction.or(Some(Direction::Forward));
        self.next = self
            .is_output(&node, sibling)
            .or_else(|| self.is_output(sibling, &node));

        if self.next.is_some() {
            return Some(node);
        }

        self.direction = self
            .direction
            .map(|dir| dir.reverse())
            .or(Some(Direction::Backward));

        self.next = self
            .is_output(&node, sibling)
            .or_else(|| self.is_output(sibling, &node));

        Some(node)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl<T, Op, Tol> Clip<'_, T, Op, Tol>
where
    T: Geometry,
    for<'a> T::Edge<'a>: Edge<'a>,
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
        while !self.next.is_some_and(|next| terminal.contains(&next)) {
            boundary.push(self.next()?.vertex);
        }

        if !self
            .direction
            .is_some_and(|direction| direction.is_forward())
        {
            boundary.reverse();
        }

        Some(boundary)
    }

    fn is_output(&self, node: &Node<T>, sibling: &Node<T>) -> Option<usize> {
        let Some(direction) = self.direction else {
            return None;
        };

        let Some(next) = self.graph.get(direction.next(node)) else {
            return None;
        };

        let (from, to) = if direction.is_forward() {
            (node, next)
        } else {
            (next, node)
        };

        let midpoint = T::Edge::new(&from.vertex, &to.vertex).midpoint();
        let corner = Corner {
            vertex: &midpoint,
            neighbors: Neighbors {
                tail: &from.vertex,
                head: &to.vertex,
            },
            role: self.graph.boundaries[node.boundary].role,
            intersection: if T::Edge::new(
                &sibling.vertex,
                &self.graph.vertices[sibling.next].vertex,
            )
            .contains(&midpoint, &self.clipper.tolerance)
            {
                Some(Intersection {
                    event: None,
                    neighbors: Neighbors {
                        tail: &sibling.vertex,
                        head: &self.graph.vertices[sibling.next].vertex,
                    },
                })
            } else if T::Edge::new(
                &self.graph.vertices[sibling.previous].vertex,
                &sibling.vertex,
            )
            .contains(&midpoint, &self.clipper.tolerance)
            {
                Some(Intersection {
                    event: None,
                    neighbors: Neighbors {
                        tail: &self.graph.vertices[sibling.previous].vertex,
                        head: &sibling.vertex,
                    },
                })
            } else {
                None
            },
        };

        Op::is_output(self.clipper.into(), corner).then(|| direction.next(node))
    }
}

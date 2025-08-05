use std::marker::PhantomData;

use crate::{
    graph::{Graph, GraphBuilder, Node}, Geometry, IsClose, Shape, Vertex
};

/// Marker for yet undefined generic parameters.
pub struct Unknown;

/// A direction to follow when traversing a boundary.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    /// Use the `next` field of the [`Node`].
    Forward,
    /// Use the `previous` field of the [`Node`].
    Backward,
}

impl Direction {
    /// Returns the index of the node following the given one.
    fn next<T>(&self, node: &Node<T>) -> usize
    where
        T: Geometry,
    {
        match self {
            Direction::Forward => node.next,
            Direction::Backward => node.previous,
        }
    }
}

/// The operation to perform by the clipping algorithm.
pub(crate) trait Operator<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, the given node belongs to the output of the clipping
    /// operation.
    fn is_output<'a>(
        ops: Operands<'a, T>,
        node: &'a Node<T>,
        tolerance: &<T::Vertex as IsClose>::Tolerance,
    ) -> bool;

    /// Returns the direction to take from the given node.
    fn direction(node: &Node<T>) -> Direction;
}

/// Implements the clipping algorithm.                                                                                                                                    
pub(crate) struct Clipper<Operator, Subject, Clip, Tolerance> {
    pub(crate) tolerance: Tolerance,
    operator: PhantomData<Operator>,
    subject: Subject,
    clip: Clip,
}

impl Default for Clipper<Unknown, Unknown, Unknown, Unknown> {
    fn default() -> Self {
        Self {
            operator: PhantomData,
            tolerance: Unknown,
            subject: Unknown,
            clip: Unknown,
        }
    }
}

impl<Op, Sub, Clip, Tol> Clipper<Op, Sub, Clip, Tol> {
    pub(crate) fn with_operator<Operator>(self) -> Clipper<Operator, Sub, Clip, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: self.clip,
        }
    }
}

impl<Op, Clip, Tol> Clipper<Op, Unknown, Clip, Tol> {
    pub(crate) fn with_subject<U>(
        self,
        subject: impl Into<Shape<U>>,
    ) -> Clipper<Op, Shape<U>, Clip, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: subject.into(),
            clip: self.clip,
        }
    }
}

impl<Op, Sub, Tol> Clipper<Op, Sub, Unknown, Tol> {
    pub(crate) fn with_clip<U>(self, clip: impl Into<Shape<U>>) -> Clipper<Op, Sub, Shape<U>, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: clip.into(),
        }
    }
}

impl<Op, Sub, Clip> Clipper<Op, Sub, Clip, Unknown> {
    pub(crate) fn with_tolerance<Tol>(self, tolerance: Tol) -> Clipper<Op, Sub, Clip, Tol> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: self.clip,
            tolerance,
        }
    }
}

impl<Op, U, Tol> Clipper<Op, Shape<U>, Shape<U>, Tol>
where
    U: Geometry + Clone + IntoIterator<Item = U::Vertex>,
    U::Vertex: IsClose<Tolerance = Tol> + Copy + PartialEq + PartialOrd,
    <U::Vertex as Vertex>::Scalar: Copy + PartialOrd,
    Op: Operator<U>,
{
    /// Performs the clipping operation and returns the resulting [`Shape`], if any.
    pub(crate) fn execute(self) -> Option<Shape<U>> {
        let mut graph = GraphBuilder::new(&self.tolerance)
            .with_subject(&self.subject)
            .with_clip(&self.clip)
            .build();

        let mut output_boundaries = Vec::new();

        let mut intersection_search = IntersectionSearch::default();
        while let Some(position) = intersection_search.next_position(&graph) {
            let Some(boundary) = BoundaryCollector::from(OutputNodes {
                graph: &mut graph,
                next: Some(position),
                direction: Direction::Forward,
                operator: PhantomData::<Op>,
            })
            .collect()
            .and_then(|boundary| U::from_raw((&self).into(), boundary, &self.tolerance)) else {
                continue;
            };

            output_boundaries.push(boundary);
        }

        let mut intersectionless_search = IntersectionlessSearch::default();
        while let Some(position) = intersectionless_search.next_position(&graph) {
            let Some(direction) = graph.nodes[position].as_ref().and_then(|node|
                Op::is_output((&self).into(), &node, &self.tolerance).then(|| Op::direction(node))
            ) else {
                continue;
            };

            let boundary = BoundaryNodes {
                graph: &mut graph,
                next: position,
                direction,
            }
            .map(|node| node.vertex)
            .collect();

            if let Some(boundary) = U::from_raw((&self).into(), boundary, &self.tolerance) {
                output_boundaries.push(boundary);
            };
        }

        if output_boundaries.is_empty() {
            return None;
        }

        Some(Shape {
            boundaries: output_boundaries,
        })
    }
}

#[derive(Debug, Default)]
struct IntersectionSearch {
    latest_position: Option<usize>,
}

impl IntersectionSearch {
    fn next_position<T>(&mut self, graph: &Graph<T>) -> Option<usize>
    where
        T: Geometry,
    {
        let start = self.latest_position.unwrap_or_default();
        self.latest_position = graph.nodes[start..]
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|node| (index, node)))
            .find(|(_, node)| node.is_intersection() && node.boundary.is_subject())
            .map(|(position, _)| position + start);

        self.latest_position
    }
}

/// An iterator of [`Node`] that yields consecutive items from a [`Graph`] which vertex belongs to
/// the output boundary.
struct OutputNodes<'a, T, Op>
where
    T: Geometry,
{
    graph: &'a mut Graph<T>,
    next: Option<usize>,
    direction: Direction,
    operator: PhantomData<Op>,
}

impl<T, Op> Iterator for OutputNodes<'_, T, Op>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        let node = self.graph.nodes[current].take()?;

        if node.is_intersection() {
            self.direction = Op::direction(&node);
        }

        let candidate = self.direction.next(&node);
        self.next = self.graph.nodes[candidate]
            .as_ref()
            .map(|next| {
                if next.is_intersection() {
                    next.siblings.first().copied()
                } else {
                    Some(candidate)
                }
            })
            .flatten();

        Some(node)
    }
}

/// A wrapper iterator around [`NodeIterator`] that stops yielding vertices when the boundary forms
/// a closed shape.
struct BoundaryCollector<'a, T, Op>
where
    T: Geometry,
{
    iterator: OutputNodes<'a, T, Op>,
    direction: Direction,
    terminal: Vec<usize>,
    closed: bool,
}

impl<'a, T, Op> From<OutputNodes<'a, T, Op>> for BoundaryCollector<'a, T, Op>
where
    T: Geometry,
{
    fn from(iterator: OutputNodes<'a, T, Op>) -> Self {
        Self {
            iterator,
            direction: Direction::Forward,
            terminal: Default::default(),
            closed: false,
        }
    }
}

impl<T, Op> Iterator for &mut BoundaryCollector<'_, T, Op>
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
            self.direction = self.iterator.direction;
            self.terminal
                .extend(node.siblings.iter().copied().chain([current]));
        } else if let Some(next) = self.iterator.next {
            self.closed = self.terminal.contains(&next);
        } else {
            self.closed = node
                .siblings
                .iter()
                .filter_map(|&sibling| self.iterator.graph.nodes[sibling].as_ref())
                .map(|sibling| sibling.next)
                .chain([node.next])
                .any(|node| self.terminal.contains(&node));
        };

        Some(node)
    }
}

impl<T, Op> BoundaryCollector<'_, T, Op>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    /// Returns a vector of vertices if, and only if, the resulting boundary forms a closed shape.
    pub(crate) fn collect(mut self) -> Option<Vec<T::Vertex>> {
        let mut vertices: Vec<_> = self.map(|node| node.vertex).collect();
        if !self.closed {
            return None;
        }

        if matches!(self.direction, Direction::Backward) {
            vertices.reverse();
        }

        Some(vertices)
    }
}

struct IntersectionlessSearch {
    next: usize,
}

impl Default for IntersectionlessSearch {
    fn default() -> Self {
        Self {
            next: Default::default(),
        }
    }
}

impl IntersectionlessSearch {
    fn next_position<T>(&mut self, graph: &Graph<T>) -> Option<usize>
    where
        T: Geometry,
    {
        if self.next >= graph.boundaries.len() {
            return None;
        }

        let (position, start) = graph.boundaries[self.next..]
            .iter()
            .enumerate()
            .find(|(_, boundary)| boundary.intersection_count == 0)
            .map(|(position, boundary)| (position + self.next, boundary.start))?;

        self.next = position + 1;
        Some(start)
    }
}

struct BoundaryNodes<'a, T>
where
    T: Geometry,
{
    graph: &'a mut Graph<T>,
    direction: Direction,
    next: usize,
}

impl<T> Iterator for BoundaryNodes<'_, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.graph.nodes[self.next]
            .take()
            .inspect(|node| self.next = self.direction.next(node))
    }
}

/// The subject and clip operands of a clipping operation.
pub struct Operands<'a, T> {
    pub subject: &'a Shape<T>,
    pub clip: &'a Shape<T>,
}

impl<'a, U, Op, Tol> From<&'a Clipper<Op, Shape<U>, Shape<U>, Tol>> for Operands<'a, U> {
    fn from(clipper: &'a Clipper<Op, Shape<U>, Shape<U>, Tol>) -> Self {
        Operands {
            subject: &clipper.subject,
            clip: &clipper.clip,
        }
    }
}

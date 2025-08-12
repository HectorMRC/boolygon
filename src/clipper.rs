use std::marker::PhantomData;

use crate::{
    graph::{Graph, GraphBuilder, Node},
    Edge, Geometry, IsClose, Shape, Vertex,
};

/// Marker for yet undefined generic parameters.
pub struct Unknown;

/// A direction to follow when traversing a boundary.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) enum Direction {
    /// Use the `next` field of the [`Node`].
    #[default]
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

    fn is_forward(&self) -> bool {
        matches!(self, Self::Forward)
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

impl<U, Op, Tol> Clipper<Op, Shape<U>, Shape<U>, Tol>
where
    U: Geometry + Clone + IntoIterator<Item = U::Vertex>,
    U::Vertex: IsClose<Tolerance = Tol> + Copy + PartialEq + PartialOrd,
    for<'a> U::Edge<'a>: Edge<'a>,
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
        let mut intersection_search = Resume::<IntersectionSearch<U>>::new(0);
        while let Some(position) = intersection_search.next(&graph) {
            let boundary = Follow::new::<Op>(&mut graph, position).collect();
            if let Some(boundary) = U::from_raw((&self).into(), boundary, &self.tolerance) {
                output_boundaries.push(boundary);
            };
        }

        let mut intersectionless_search = Resume::<IntersectionlessSearch<U>>::new(0);
        while let Some(position) = intersectionless_search.next(&graph) {
            let Some(node) = &graph.nodes[position] else {
                continue;
            };

            // Dodge pseudo-intersection nodes by taking the midpoint.
            let node = if node.is_pseudo_intersection
                && let Some(next) = &graph.nodes[node.next]
            {
                &Node {
                    vertex: U::Edge::new(&node.vertex, &next.vertex).midpoint(),
                    previous: position,
                    intersection: None,
                    is_pseudo_intersection: false,
                    siblings: Default::default(),
                    ..*node
                }
            } else {
                node
            };

            if !Op::is_output((&self).into(), node, &self.tolerance) {
                continue;
            };

            let boundary = Drain::new(&mut graph, position).collect::<Op>();
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

/// An iterator whose state that can be restored.
trait Restorable: Iterator<Item = usize> {
    type Geometry: Geometry;
    type State;

    /// Restores the iterator with the given [`Graph`] and state.
    fn restore(
        graph: &Graph<Self::Geometry>,
        state: Self::State,
    ) -> impl Restorable<State = Self::State>;
    fn state(&self) -> Self::State;
}

/// A wrapper around iterators that captures their state and restores it.
struct Resume<I>
where
    I: Restorable,
{
    iterator: PhantomData<I>,
    state: I::State,
}

impl<I> Resume<I>
where
    I: Restorable,
    I::State: Copy,
{
    /// Returns the next item from the restorable iterator.
    fn next(&mut self, graph: &Graph<I::Geometry>) -> Option<usize> {
        let mut iterator = I::restore(graph, self.state);
        let current = iterator.next();
        self.state = iterator.state();

        current
    }
}

impl<I> Resume<I>
where
    I: Restorable,
{
    /// Returns a new resumable iterator with the given initial state.
    fn new(state: I::State) -> Self {
        Self {
            iterator: PhantomData,
            state,
        }
    }
}

/// Searches for subject intersections in the [`Graph`].
struct IntersectionSearch<'a, T>
where
    T: Geometry,
{
    graph: &'a Graph<T>,
    next: usize,
}

impl<'a, T> Iterator for IntersectionSearch<'a, T>
where
    T: Geometry,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (position, node) = self
            .graph
            .nodes
            .get(self.next..)?
            .iter()
            .enumerate()
            .filter_map(|(position, node)| node.as_ref().map(|node| (position, node)))
            .find(|(_, node)| node.is_intersection())
            .map(|(position, node)| (position + self.next, node))?;

        self.next = position + 1;

        if !node.is_intersection() {
            return self.next();
        }

        Some(position)
    }
}

impl<T> Restorable for IntersectionSearch<'_, T>
where
    T: Geometry,
{
    type Geometry = T;
    type State = usize;

    fn restore(
        graph: &Graph<Self::Geometry>,
        next: Self::State,
    ) -> impl Restorable<State = Self::State> {
        IntersectionSearch { graph, next }
    }

    fn state(&self) -> Self::State {
        self.next
    }
}

/// Yields each [`Node`] from the [`Graph`] within the path starting at the given position.
struct Follow<'a, T, Op>
where
    T: Geometry,
{
    graph: &'a mut Graph<T>,
    next: Option<usize>,
    direction: Direction,
    operator: PhantomData<Op>,
    terminal: Vec<usize>,
    closed: bool,
}

impl<T, Op> Iterator for Follow<'_, T, Op>
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

        let current = self.next?;
        let node = self.graph.nodes[current].take()?;

        if node.is_intersection() {
            self.direction = Op::direction(&node);
        }

        let candidate = self.direction.next(&node);
        self.next = self.graph.nodes[candidate].as_mut().and_then(|next| {
            if next.is_intersection() {
                next.siblings.pop_first()
            } else {
                Some(candidate)
            }
        });

        if self.terminal.is_empty() {
            self.terminal
                .extend(node.siblings.iter().copied().chain([current]));
        } else if let Some(next) = self.next {
            self.closed = self.terminal.contains(&next);
        } else {
            self.closed = node
                .siblings
                .iter()
                .filter_map(|&sibling| self.graph.nodes[sibling].as_ref())
                .map(|sibling| sibling.next)
                .chain([self.direction.next(&node)])
                .any(|node| self.terminal.contains(&node));
        };

        Some(node)
    }
}

impl<T, Op> Follow<'_, T, Op>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    /// Returns the full path yielded by this iterator.
    fn collect(self) -> Vec<T::Vertex> {
        let orientation = self
            .next
            .and_then(|position| self.graph.nodes[position].as_ref())
            .map(|node| Op::direction(node))
            .unwrap_or_default();
        let mut boundary = self.map(|node| node.vertex).collect::<Vec<_>>();

        if !orientation.is_forward() {
            boundary.reverse();
        }

        boundary
    }
}

impl<'a, T> Follow<'a, T, Unknown>
where
    T: Geometry,
{
    /// Returns a new iterator that begins at the given position.
    fn new<Op>(graph: &'a mut Graph<T>, start: usize) -> Follow<'a, T, Op> {
        Follow {
            graph,
            next: Some(start),
            direction: Direction::Forward,
            operator: PhantomData::<Op>,
            terminal: Default::default(),
            closed: false,
        }
    }
}

/// Searches for the first [`Node`] in the [`Graph`] belonging to a geometry with no intersections.
struct IntersectionlessSearch<'a, T>
where
    T: Geometry,
{
    graph: &'a Graph<T>,
    next: usize,
}

impl<'a, T> Iterator for IntersectionlessSearch<'a, T>
where
    T: Geometry,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (position, start) = self
            .graph
            .boundaries
            .get(self.next..)?
            .iter()
            .enumerate()
            .find(|(_, boundary)| boundary.intersection_count == 0)
            .map(|(position, boundary)| (position + self.next, boundary.start))?;

        self.next = position + 1;
        Some(start)
    }
}

impl<T> Restorable for IntersectionlessSearch<'_, T>
where
    T: Geometry,
{
    type Geometry = T;
    type State = usize;

    fn restore(
        graph: &Graph<Self::Geometry>,
        next: Self::State,
    ) -> impl Restorable<State = Self::State> {
        IntersectionlessSearch { graph, next }
    }

    fn state(&self) -> Self::State {
        self.next
    }
}

/// Yields all the nodes from a boundary that never intersects.
struct Drain<'a, T>
where
    T: Geometry,
{
    graph: &'a mut Graph<T>,
    next: Option<usize>,
    start: usize,
}

impl<'a, T> Iterator for Drain<'a, T>
where
    T: Geometry,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next
            && current == self.start
        {
            return None;
        }

        let current = self.next.unwrap_or(self.start);
        let node = self.graph.nodes[current].take()?;
        self.next = Some(node.next);

        Some(node)
    }
}

impl<T> Drain<'_, T>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
{
    /// Returns the full path yielded by this iterator.
    fn collect<Op>(self) -> Vec<T::Vertex>
    where
        Op: Operator<T>,
    {
        let orientation = self.graph.nodes[self.start]
            .as_ref()
            .map(|node| Op::direction(node))
            .unwrap_or_default();

        let mut boundary = self.map(|node| node.vertex).collect::<Vec<_>>();

        if !orientation.is_forward() {
            boundary.reverse();
        }

        boundary
    }
}

impl<'a, T> Drain<'a, T>
where
    T: Geometry,
{
    fn new(graph: &'a mut Graph<T>, start: usize) -> Self {
        Self {
            graph,
            next: None,
            start,
        }
    }
}

/// The subject and clip operands of a clipping operation.
#[derive(Debug, Clone, Copy)]
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

use std::{
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

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
    fn is_output(ctx: Context<'_, T>, node: &Node<T>) -> bool;

    /// Returns the direction to take from the given node.
    fn direction(ctx: Context<'_, T>, node: &Node<T>) -> Direction;
}

/// Implements the clipping algorithm.                                                                                                                                    
pub(crate) struct Clipper<Subject, Clip, Operator, Tolerance> {
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

impl<Sub, Clip, Op, Tol> Clipper<Sub, Clip, Op, Tol> {
    pub(crate) fn with_operator<Operator>(self) -> Clipper<Sub, Clip, Operator, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: self.clip,
        }
    }
}

impl<Clip, Op, Tol> Clipper<Unknown, Clip, Op, Tol> {
    pub(crate) fn with_subject<U>(
        self,
        subject: impl Into<Shape<U>>,
    ) -> Clipper<Shape<U>, Clip, Op, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: subject.into(),
            clip: self.clip,
        }
    }
}

impl<Sub, Op, Tol> Clipper<Sub, Unknown, Op, Tol> {
    pub(crate) fn with_clip<U>(self, clip: impl Into<Shape<U>>) -> Clipper<Sub, Shape<U>, Op, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: clip.into(),
        }
    }
}

impl<Sub, Clip, Op> Clipper<Sub, Clip, Op, Unknown> {
    pub(crate) fn with_tolerance<Tol>(self, tolerance: Tol) -> Clipper<Sub, Clip, Op, Tol> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: self.clip,
            tolerance,
        }
    }
}

impl<T, Op, Tol> Clipper<Shape<T>, Shape<T>, Op, Tol>
where
    T: Geometry + Clone + IntoIterator<Item = T::Vertex>,
    T::Vertex: IsClose<Tolerance = Tol> + Copy + PartialEq + PartialOrd,
    for<'a> T::Edge<'a>: Edge<'a>,
    <T::Vertex as Vertex>::Scalar: Copy + PartialOrd,
    Op: Operator<T>,
{
    /// Performs the clipping operation and returns the resulting [`Shape`], if any.
    pub(crate) fn execute(self) -> Option<Shape<T>> {
        let mut graph = GraphBuilder::new(&self.tolerance)
            .with_subject(&self.subject)
            .with_clip(&self.clip)
            .build();

        let mut output_boundaries = Vec::new();
        let mut intersection_search = Resume::<IntersectionSearch<T>>::new(0);
        while let Some(position) = intersection_search.next(&graph) {
            if let Some(boundary) = self.follow(&mut graph, position).collect()
                && let Some(boundary) = T::from_raw((&self).into(), boundary, &self.tolerance)
            {
                output_boundaries.push(boundary);
            };
        }

        let mut intersectionless_search = Resume::<IntersectionlessSearch<T>>::new(0);
        while let Some(position) = intersectionless_search.next(&graph) {
            if let Some(node) = &graph.nodes[position]
                && Op::is_output((&self).into(), node)
            {
                let boundary = self.drain(&mut graph, position).collect::<Op>();
                if let Some(boundary) = T::from_raw((&self).into(), boundary, &self.tolerance) {
                    output_boundaries.push(boundary);
                };
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

impl<'a, T, Op, Tol> Clipper<Shape<T>, Shape<T>, Op, Tol>
where
    T: Geometry,
    T::Vertex: IsClose<Tolerance = Tol>,
{
    fn follow(&'a self, graph: &'a mut Graph<T>, start: usize) -> Follow<'a, T, Op> {
        Follow {
            graph,
            next: Some(start),
            direction: Direction::Forward,
            operator: PhantomData::<Op>,
            terminal: Vec::with_capacity(2),
            context: self.into(),
            closed: Default::default(),
        }
    }

    fn drain(&'a self, graph: &'a mut Graph<T>, start: usize) -> Drain<'a, T> {
        Drain {
            graph,
            context: self.into(),
            next: None,
            start,
        }
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
        let position = self
            .graph
            .nodes
            .get(self.next..)?
            .iter()
            .enumerate()
            .filter_map(|(position, node)| Some((position, node.as_ref()?)))
            .find(|(_, node)| node.boundary.is_subject() && node.intersection.is_some())
            .map(|(position, _)| position + self.next)?;

        self.next = position + 1;
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
    direction: Direction,
    next: Option<usize>,
    terminal: Vec<usize>,
    closed: Rc<AtomicBool>,
    context: Context<'a, T>,
    operator: PhantomData<Op>,
}

impl<T, Op> Iterator for Follow<'_, T, Op>
where
    T: Geometry,
    T::Vertex: Copy + PartialEq,
    Op: Operator<T>,
{
    type Item = Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.closed.load(Ordering::Relaxed) {
            return None;
        }

        let current = self.next?;
        let node = self.graph.nodes[current].take()?;

        if node.intersection.is_some() {
            self.direction = Op::direction(self.context, &node);
        }

        let candidate = self.direction.next(&node);
        self.next = self.graph.nodes[candidate].as_mut().and_then(|next| {
            next.intersection
                .as_ref()
                .map(|intersection| intersection.sibling)
                .or(Some(candidate))
        });

        if self.terminal.is_empty() {
            self.terminal.extend(
                node.intersection
                    .iter()
                    .map(|intersection| intersection.sibling)
                    .chain([current]),
            );
        } else if let Some(next) = self.next {
            self.closed
                .store(self.terminal.contains(&next), Ordering::Relaxed);
        } else {
            self.closed.store(
                node.intersection
                    .iter()
                    .filter_map(|intersection| self.graph.nodes[intersection.sibling].as_ref())
                    .map(|sibling| sibling.next)
                    .chain([self.direction.next(&node)])
                    .any(|node| self.terminal.contains(&node)),
                Ordering::Relaxed,
            );
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
    fn collect(self) -> Option<Vec<T::Vertex>> {
        let orientation = self
            .next
            .and_then(|position| self.graph.nodes[position].as_ref())
            .map(|node| Op::direction(self.context, node))
            .unwrap_or_default();

        let is_closed = self.closed.clone();
        let mut boundary = self.map(|node| node.vertex).collect::<Vec<_>>();

        if !is_closed.load(Ordering::Relaxed) {
            return None;
        }

        if !orientation.is_forward() {
            boundary.reverse();
        }

        Some(boundary)
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
    context: Context<'a, T>,
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
            .map(|node| Op::direction(self.context, node))
            .unwrap_or_default();

        let mut boundary = self.map(|node| node.vertex).collect::<Vec<_>>();

        if !orientation.is_forward() {
            boundary.reverse();
        }

        boundary
    }
}

/// The context of a clipping operation.
pub struct Context<'a, T>
where
    T: Geometry,
{
    /// The shape being clipped in this operation.
    pub subject: &'a Shape<T>,
    /// The clip shape involved in this operation.
    pub clip: &'a Shape<T>,
    /// The tolerance being used in this operation.
    pub tolerance: &'a <T::Vertex as IsClose>::Tolerance,
}

impl<T> Copy for Context<'_, T> where T: Geometry {}
impl<T> Clone for Context<'_, T>
where
    T: Geometry,
{
    fn clone(&self) -> Self {
        Self {
            subject: self.subject,
            clip: self.clip,
            tolerance: self.tolerance,
        }
    }
}

impl<'a, T, Op, Tol> From<&'a Clipper<Shape<T>, Shape<T>, Op, Tol>> for Context<'a, T>
where
    T: Geometry,
    T::Vertex: IsClose<Tolerance = Tol>,
{
    fn from(clipper: &'a Clipper<Shape<T>, Shape<T>, Op, Tol>) -> Self {
        Context {
            subject: &clipper.subject,
            clip: &clipper.clip,
            tolerance: &clipper.tolerance,
        }
    }
}

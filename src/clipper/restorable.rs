use std::marker::PhantomData;

use crate::{graph::Graph, Geometry};

/// An iterator whose state that can be restored.
pub(super) trait Restorable: Iterator<Item = usize> {
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
pub(super) struct Resume<I>
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
    pub(super) fn next(&mut self, graph: &Graph<I::Geometry>) -> Option<usize> {
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
    pub(super) fn new(state: I::State) -> Self {
        Self {
            iterator: PhantomData,
            state,
        }
    }
}

/// Searches for subject intersections in the [`Graph`].
pub(super) struct IntersectionSearch<'a, T>
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
            .vertices
            .get(self.next..)?
            .iter()
            .enumerate()
            .filter(|(_, vertex)| !vertex.visited)
            .filter(|(_, vertex)| self.graph.boundaries[vertex.boundary].role.is_subject())
            .find(|(_, vertex)| vertex.intersection.is_some())
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


/// Searches for the first [`Node`] in the [`Graph`] belonging to a boundary that has not been visited.
pub(super) struct UnvisitedSearch<'a, T>
where
    T: Geometry,
{
    graph: &'a Graph<T>,
    next: usize,
}

impl<'a, T> Iterator for UnvisitedSearch<'a, T>
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
            .find(|(_, boundary)| !boundary.visited)
            .map(|(position, boundary)| (position + self.next, boundary.entrypoint))?;

        self.next = position + 1;
        Some(start)
    }
}

impl<T> Restorable for UnvisitedSearch<'_, T>
where
    T: Geometry,
{
    type Geometry = T;
    type State = usize;

    fn restore(
        graph: &Graph<Self::Geometry>,
        next: Self::State,
    ) -> impl Restorable<State = Self::State> {
        UnvisitedSearch { graph, next }
    }

    fn state(&self) -> Self::State {
        self.next
    }
}
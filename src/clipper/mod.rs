mod restorable;
mod context;
mod clip;
mod traverse;

pub use context::{Context};

use std::{marker::PhantomData};

use self::restorable::{Resume, IntersectionSearch, UnvisitedSearch};
use self::clip::Clip;
use self::traverse::Traverse;

use crate::{
    graph::{Graph, Node}, Corner, Edge, Geometry, IsClose, Neighbors, Shape, Vertex
};

/// Marker for yet undefined generic parameters.
pub struct Unknown;

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
    fn is_output(ctx: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> bool;

    /// Returns the direction to take from the given node.
    fn direction(ctx: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> Direction;
}

/// Implements the clipping algorithm.                                                                                                                                    
pub(crate) struct Clipper<Subject, Clip, Operator, Tolerance> {
    subject: Subject,
    clip: Clip,
    tolerance: Tolerance,
    operator: PhantomData<Operator>,
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
    T: Geometry,
    for<'a> &'a T: IntoIterator<Item = &'a T::Vertex>,
    for<'a> T::Edge<'a>: Edge<'a>,
    T::Vertex: IsClose<Tolerance = Tol> + Copy + PartialEq + PartialOrd,
    <T::Vertex as Vertex>::Scalar: Copy + PartialOrd,
    Op: Operator<T>,
{
    /// Performs the clipping operation and returns the resulting [`Shape`], if any.
    pub(crate) fn execute(self) -> Option<Shape<T>> {
        let mut graph = Graph::builder(&self.tolerance)
            .with_subject(&self.subject)
            .with_clip(&self.clip)
            .build();
        
        let mut output_boundaries = Vec::new();
        let mut intersection_search = Resume::<IntersectionSearch<T>>::new(0);
        while let Some(position) = intersection_search.next(&graph) {
            if let Some(boundary) = self.clip(&mut graph, position).collect()
                && let Some(boundary) = T::from_raw((&self).into(), boundary, &self.tolerance)
            {
                output_boundaries.push(boundary);
            };
        }

        let mut intersectionless_search = Resume::<UnvisitedSearch<T>>::new(0);
        while let Some(position) = intersectionless_search.next(&graph) {
            let node = &graph.vertices[position];
            let next = &graph.vertices[node.next];
            let corner = Corner { 
                vertex: &T::Edge::new(&node.vertex, &next.vertex).midpoint(), 
                neighbors: Neighbors { tail: &node.vertex, head: &next.vertex }, 
                role: graph.boundaries[node.boundary].role, 
                intersection: None
            };
            
            if Op::is_output((&self).into(), corner) {
                let boundary = self.traverse(&mut graph, position).collect();
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
    fn clip(&'a self, graph: &'a mut Graph<T>, start: usize) -> Clip<'a, T, Op, Tol> {
        Clip {
            clipper: self,
            graph,
            direction: Direction::Forward,
            next: None,
            start,
        }
    }

    fn traverse(&'a self, graph: &'a mut Graph<T>, start: usize) -> Traverse<'a, T, Op, Tol> {
        Traverse {
            clipper: self,
            graph,
            next: None,
            start,
        }
    }
}

mod graph;
mod vertex;

use std::{fmt::Debug, marker::PhantomData};

use num_traits::{Float, Signed};

use crate::shape::Shape;

use self::graph::GraphBuilder;
use self::vertex::VerticesIterator;
pub use self::vertex::{Role, Vertex};

pub struct Unknown;

pub(crate) struct Operands<'a, T> {
    pub subject: &'a Shape<T>,
    pub clip: &'a Shape<T>,
}

impl<'a, T, Op> From<&'a Clipper<Op, Shape<T>, Shape<T>>> for Operands<'a, T> {
    fn from(clipper: &'a Clipper<Op, Shape<T>, Shape<T>>) -> Self {
        Operands {
            subject: &clipper.subject,
            clip: &clipper.clip,
        }
    }
}

/// Represents the operation to perform by the clipping algorithm.
pub(crate) trait Operator<T> {
    /// Returns true if, and only if, the given vertex belongs to the output of the clipping
    /// operation.
    fn is_output<'a>(ops: Operands<'a, T>, vertex: &'a Vertex<T>) -> bool;
}

/// Implements the clipping algorithm.                                                                                                                                   
pub(crate) struct Clipper<Operator, Subject, Clip> {
    operator: PhantomData<Operator>,
    subject: Subject,
    clip: Clip,
}

impl Default for Clipper<Unknown, Unknown, Unknown> {
    fn default() -> Self {
        Self {
            operator: PhantomData,
            subject: Unknown,
            clip: Unknown,
        }
    }
}

impl<Op, Sub, Clip> Clipper<Op, Sub, Clip> {
    pub fn with_operator<Operator>(self) -> Clipper<Operator, Sub, Clip> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: self.clip,
        }
    }
}

impl<Op, Clip> Clipper<Op, Unknown, Clip> {
    pub fn with_subject<T>(self, subject: impl Into<Shape<T>>) -> Clipper<Op, Shape<T>, Clip> {
        Clipper {
            operator: PhantomData,
            subject: subject.into(),
            clip: self.clip,
        }
    }
}

impl<Op, Sub> Clipper<Op, Sub, Unknown> {
    pub fn with_clip<T>(self, clip: impl Into<Shape<T>>) -> Clipper<Op, Sub, Shape<T>> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: clip.into(),
        }
    }
}

impl<T, Op> Clipper<Op, Shape<T>, Shape<T>>
where
    T: Clone + PartialOrd + Signed + Float + Debug,
    Op: Operator<T>,
{
    pub fn execute(self) -> Option<Shape<T>> {
        let mut graph = GraphBuilder::default()
            .with_subject(self.subject.clone())
            .with_clip(self.clip.clone())
            .build();

        let mut output = None;
        let mut store_output = |polygon| {
            match output.as_mut() {
                None => output = Some(Shape::from(polygon)),
                Some(shape) => shape.polygons.push(polygon),
            };
        };

        while let Some(iter) = graph
            .position_where(Vertex::is_intersection)
            .map(|position| VerticesIterator {
                clipper: &self,
                graph: &mut graph,
                next: position,
            })
        {
            store_output(iter.map(|vertex| vertex.point).collect::<Vec<_>>().into());
        }

        while let Some(iter) = graph
            .position_where(|vertex| Op::is_output((&self).into(), vertex))
            .map(|position| VerticesIterator {
                clipper: &self,
                graph: &mut graph,
                next: position,
            })
        {
            let start = iter.next;
            let vertices = iter.collect::<Vec<_>>();

            if vertices[vertices.len() - 1].next != start {
                // The succession of vertices is an open shape.
                continue;
            }

            store_output(
                vertices
                    .into_iter()
                    .map(|vertex| vertex.point)
                    .collect::<Vec<_>>()
                    .into(),
            );
        }

        output
    }
}

use std::marker::PhantomData;

use crate::{
    graph::{Graph, GraphBuilder},
    node::{Node, NodeIterator},
    Edge, Geometry, IsClose, Shape, Tolerance,
};

/// Marker for yet undefined generic parameters.
pub struct Unknown;

/// References to both operand, the subject and clip shapes, of a clipping operation.
pub struct Operands<'a, T> {
    pub subject: &'a Shape<T>,
    pub clip: &'a Shape<T>,
}

impl<'a, T, U, Op> From<&'a Clipper<T, Op, Shape<U>, Shape<U>>> for Operands<'a, U> {
    fn from(clipper: &'a Clipper<T, Op, Shape<U>, Shape<U>>) -> Self {
        Operands {
            subject: &clipper.subject,
            clip: &clipper.clip,
        }
    }
}

/// Represents the operation to perform by the clipping algorithm.
pub(super) trait Operator<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, the given node belongs to the output of the clipping
    /// operation.
    fn is_output<'a>(
        ops: Operands<'a, T>,
        node: &'a Node<T>,
        tolerance: &Tolerance<<T::Vertex as IsClose>::Scalar>,
    ) -> bool;
}

/// Implements the clipping algorithm.                                                                                                                                   
pub(super) struct Clipper<T, Operator, Subject, Clip> {
    pub(super) tolerance: Tolerance<T>,
    operator: PhantomData<Operator>,
    subject: Subject,
    clip: Clip,
}

impl<T> Clipper<T, Unknown, Unknown, Unknown> {
    /// Returns a default clipper with the given tolerance.
    pub fn new(tolerance: Tolerance<T>) -> Self {
        Self {
            operator: PhantomData,
            subject: Unknown,
            clip: Unknown,
            tolerance,
        }
    }
}

impl<T, Op, Sub, Clip> Clipper<T, Op, Sub, Clip> {
    pub fn with_operator<Operator>(self) -> Clipper<T, Operator, Sub, Clip> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: self.clip,
            tolerance: self.tolerance,
        }
    }
}

impl<T, Op, Clip> Clipper<T, Op, Unknown, Clip> {
    pub fn with_subject<U>(self, subject: impl Into<Shape<U>>) -> Clipper<T, Op, Shape<U>, Clip> {
        Clipper {
            operator: PhantomData,
            subject: subject.into(),
            clip: self.clip,
            tolerance: self.tolerance,
        }
    }
}

impl<T, Op, Sub> Clipper<T, Op, Sub, Unknown> {
    pub fn with_clip<U>(self, clip: impl Into<Shape<U>>) -> Clipper<T, Op, Sub, Shape<U>> {
        Clipper {
            operator: PhantomData,
            subject: self.subject,
            clip: clip.into(),
            tolerance: self.tolerance,
        }
    }
}

impl<T, U, Op> Clipper<T, Op, Shape<U>, Shape<U>>
where
    T: Copy + PartialOrd,
    U: Geometry + Clone + IntoIterator<Item = U::Vertex>,
    U::Vertex: IsClose<Scalar = T> + Copy + PartialEq + PartialOrd,
    Op: Operator<U>,
{
    /// Performs the clipping operation and returns the resulting [`Shape`], if any.
    pub fn execute(self) -> Option<Shape<U>> {
        let mut graph = GraphBuilder::new(self.tolerance)
            .with_subject(self.subject.clone())
            .with_clip(self.clip.clone())
            .build();

        let mut output = None;
        while let Some(position) =
            graph.position_where(|node| Op::is_output((&self).into(), node, &self.tolerance))
        {
            // By starting at the next node it is ensured there is a path to follow.
            let Some(position) = self.select_path(&graph, graph.nodes[position].as_ref()?) else {
                graph.purge(position);
                continue;
            };

            let nodes = NodeIterator {
                clipper: &self,
                graph: &mut graph,
                init: position,
                next: None,
            }
            .map(|node| node.vertex)
            .collect();

            let Some(polygon) = U::from_raw((&self).into(), nodes, &self.tolerance) else {
                continue;
            };

            match output.as_mut() {
                None => output = Some(Shape::new(polygon)),
                Some(shape) => shape.polygons.push(polygon),
            };
        }

        output
    }
}

impl<T, U, Op> Clipper<T, Op, Shape<U>, Shape<U>>
where
    T: Copy,
    U: Geometry,
    U::Vertex: IsClose<Scalar = T>,
    Op: Operator<U>,
{
    pub(super) fn select_path(&self, graph: &Graph<U>, node: &Node<U>) -> Option<usize> {
        node.siblings
            .iter()
            .filter_map(|&sibling| graph.nodes[sibling].as_ref())
            .chain([node])
            .rev()
            .find_map(|target| {
                graph.nodes[target.next]
                    .as_ref()
                    .is_some_and(|next| {
                        let subject = if next.is_intersection() {
                            &Node {
                                vertex: U::Edge::new(&target.vertex, &next.vertex).midpoint(),
                                role: next.role,
                                previous: Default::default(),
                                next: Default::default(),
                                siblings: Default::default(),
                            }
                        } else {
                            next
                        };

                        Op::is_output(self.into(), subject, &self.tolerance)
                    })
                    .then_some(target.next)
            })
    }
}

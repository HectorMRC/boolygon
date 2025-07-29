use std::marker::PhantomData;

use crate::{
    graph::{Graph, GraphBuilder},
    node::{BoundaryCollector, Node, NodeIterator},
    Edge, Geometry, IsClose, Shape, Vertex,
};

/// Marker for yet undefined generic parameters.
pub struct Unknown;

/// References to both operand, the subject and clip shapes, of a clipping operation.
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
        tolerance: &<T::Vertex as IsClose>::Tolerance,
    ) -> bool;
}

/// Implements the clipping algorithm.                                                                                                                                   
pub(super) struct Clipper<Operator, Subject, Clip, Tolerance> {
    pub(super) tolerance: Tolerance,
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
    pub(super) fn with_operator<Operator>(self) -> Clipper<Operator, Sub, Clip, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: self.clip,
        }
    }
}

impl<Op, Clip, Tol> Clipper<Op, Unknown, Clip, Tol> {
    pub(super) fn with_subject<U>(self, subject: impl Into<Shape<U>>) -> Clipper<Op, Shape<U>, Clip, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: subject.into(),
            clip: self.clip,
        }
    }
}

impl<Op, Sub, Tol> Clipper<Op, Sub, Unknown, Tol> {
    pub(super) fn with_clip<U>(self, clip: impl Into<Shape<U>>) -> Clipper<Op, Sub, Shape<U>, Tol> {
        Clipper {
            operator: PhantomData,
            tolerance: self.tolerance,
            subject: self.subject,
            clip: clip.into(),
        }
    }
}

impl<Op, Sub, Clip> Clipper<Op, Sub, Clip, Unknown> {
    pub(super) fn with_tolerance<Tol>(self, tolerance: Tol) -> Clipper<Op, Sub, Clip, Tol> {
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
    <U::Vertex as Vertex>::Scalar: Copy + PartialOrd,
    Op: Operator<U>,
{
    /// Performs the clipping operation and returns the resulting [`Shape`], if any.
    pub(super) fn execute(self) -> Option<Shape<U>> {
        let mut graph = GraphBuilder::new(&self.tolerance)
            .with_subject(self.subject.clone())
            .with_clip(self.clip.clone())
            .build();

        let mut output = None;
        while let Some(position) =
            graph.position_where(|node| Op::is_output((&self).into(), node, &self.tolerance))
        {
            let Some(nodes) = BoundaryCollector::from(NodeIterator {
                clipper: &self,
                graph: &mut graph,
                next: Some(position),
            })
            .collect() else {
                continue;
            };

            let Some(boundary) = U::from_raw((&self).into(), nodes, &self.tolerance) else {
                continue;
            };

            match output.as_mut() {
                None => output = Some(Shape::new(boundary)),
                Some(shape) => shape.boundaries.push(boundary),
            };
        }

        output
    }
}

impl<U, Op, Tol> Clipper<Op, Shape<U>, Shape<U>, Tol>
where
    U: Geometry,
    U::Vertex: IsClose<Tolerance = Tol>,
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

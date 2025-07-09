use std::fmt::Debug;

use num_traits::{Float, Signed};

use crate::{
    cartesian::{Point, Segment},
    shape::Shape,
};

use super::{graph::Graph, Clipper, Operator};

/// Determines the role of a [`Vertex`] during the clipping process.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Role {
    /// The vertex belongs to the subject shape.
    Subject,
    /// The vertex belongs to the clip shape.
    Clip,
}

#[derive(Debug)]
pub(crate) struct Vertex<T> {
    /// The location of the vertex.
    pub(crate) point: Point<T>,
    /// The role of the vertex.
    pub(crate) role: Role,
    /// The index of the vertex following this one.
    pub(super) next: usize,
    /// The index of the vertex previous to this one.
    pub(super) previous: usize,
    /// Vertices from the oposite shape located at the same point.
    pub(super) siblings: Vec<usize>,
}

impl<T> Vertex<T> {
    pub(super) fn is_intersection(&self) -> bool {
        !self.siblings.is_empty()
    }

    pub(super) fn is_subject(&self) -> bool {
        matches!(self.role, Role::Subject)
    }
}

pub(super) struct VerticesIterator<'a, Op, T> {
    pub(super) clipper: &'a Clipper<Op, Shape<T>, Shape<T>>,
    pub(super) graph: &'a mut Graph<T>,
    pub(super) next: Option<usize>,
}

impl<Op, T> Iterator for VerticesIterator<'_, Op, T>
where
    T: Signed + Float + Debug,
    Op: Operator<T>,
{
    type Item = Vertex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let vertex = self.graph.vertices[self.next?].take()?;

        self.next = if vertex.is_intersection() {
            self.select_sibling(&vertex)
        } else {
            Some(vertex.next)
        };

        Some(vertex)
    }
}

impl<Op, T> VerticesIterator<'_, Op, T>
where
    T: Signed + Float + Debug,
    Op: Operator<T>,
{
    fn select_sibling(&mut self, vertex: &Vertex<T>) -> Option<usize> {
        vertex.siblings.iter().find_map(|&sibling| {
            self.graph.vertices[sibling]
                .as_ref()
                .and_then(|sibling| {
                    self.graph.vertices[sibling.next]
                        .as_ref()
                        .map(|next| (sibling, next))
                })
                .is_some_and(|(sibling, next)| {
                    let subject = if next.is_intersection() {
                        &Vertex {
                            point: Segment::from((&sibling.point, &next.point)).midpoint(),
                            role: next.role,
                            previous: Default::default(),
                            next: Default::default(),
                            siblings: Default::default(),
                        }
                    } else {
                        next
                    };

                    Op::is_output(self.clipper.into(), subject)
                })
                .then(|| {
                    self.graph.vertices[sibling]
                        .take()
                        .map(|vertex| vertex.next)
                })
                .flatten()
        })
    }
}

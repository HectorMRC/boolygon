use std::{fmt::Debug, marker::PhantomData};

use crate::{
    Corner, Role, 
    clipper::{Clipper, Direction, Operator}, Context, Edge, Event, Geometry, IsClose, Vertex
};

/// A combination of disjoint boundaries.
#[derive(Debug, Clone)]
pub struct Shape<T> {
    /// The list of non-crossing boundaries.
    pub(crate) boundaries: Vec<T>,
}

impl<T> From<T> for Shape<T>
where
    T: Geometry,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> PartialEq for Shape<T>
where
    T: PartialEq + Clone,
{
    fn eq(&self, other: &Self) -> bool {
        if self.boundaries.len() != other.boundaries.len() {
            return false;
        }

        self.boundaries
            .iter()
            .all(|a| other.boundaries.iter().any(|b| a.eq(b)))
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    for<'a> &'a T: IntoIterator<Item = &'a T::Vertex>,
    for<'a> T::Edge<'a>: Edge<'a>,
    T::Vertex: Copy + PartialEq + PartialOrd,
    <T::Vertex as Vertex>::Scalar: Copy + PartialOrd,
{
    /// Returns the union of this shape and the other.
    pub fn or(self, other: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Option<Self> {
        struct OrOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for OrOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(ctx: Context<'a, T>, corner: Corner<'_, T::Vertex>) -> bool {
                match corner.role {
                    Role::Subject => !ctx.operands.clip.contains(&corner.vertex, ctx.tolerance),
                    Role::Clip => !ctx.operands.subject.contains(&corner.vertex, ctx.tolerance),
                }
            }

            fn direction(_: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> Direction {
                let Some(intersection) = corner
                    .intersection
                    .as_ref()
                    .and_then(|intersection| intersection.event)
                else {
                    return Direction::Forward;
                };

                match intersection {
                    Event::Entry => Direction::Backward,
                    Event::Exit => Direction::Forward,
                }
            }
        }

        Clipper::default()
            .with_operator::<OrOperator<T>>()
            .with_tolerance(tolerance)
            .with_subject(self)
            .with_clip(other)
            .execute()
    }

    /// Returns the difference of the other shape on this one.
    pub fn not(self, other: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Option<Self> {
        struct NotOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for NotOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(ctx: Context<'a, T>, corner: Corner<'_, T::Vertex>) -> bool {
                match corner.role {
                    Role::Subject => !ctx.operands.clip.contains(&corner.vertex, ctx.tolerance),
                    Role::Clip => ctx.operands.subject.contains(&corner.vertex, ctx.tolerance),
                }
            }

            fn direction(_: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> Direction {
                let Some(intersection) = corner
                    .intersection
                    .as_ref()
                    .and_then(|intersection| intersection.event)
                else {
                    return if corner.role.is_subject() {
                        Direction::Forward
                    } else {
                        Direction::Backward
                    };
                };

                match (corner.role, intersection) {
                    (Role::Subject, Event::Entry) => Direction::Backward,
                    (Role::Subject, Event::Exit) => Direction::Forward,
                    (Role::Clip, Event::Entry) => Direction::Forward,
                    (Role::Clip, Event::Exit) => Direction::Backward,
                }
            }
        }

        Clipper::default()
            .with_operator::<NotOperator<T>>()
            .with_tolerance(tolerance)
            .with_clip(other)
            .with_subject(self)
            .execute()
    }

    /// Returns the intersection of this shape and the other.
    pub fn and(self, other: Self, tolerance: <T::Vertex as IsClose>::Tolerance) -> Option<Self> {
        struct AndOperator<T>(PhantomData<T>);

        impl<T> Operator<T> for AndOperator<T>
        where
            T: Geometry,
        {
            fn is_output<'a>(ctx: Context<'a, T>, corner: Corner<'_, T::Vertex>) -> bool {
                match corner.role {
                    Role::Subject => ctx.operands.clip.contains(&corner.vertex, ctx.tolerance),
                    Role::Clip => ctx.operands.subject.contains(&corner.vertex, ctx.tolerance),
                }
            }

            fn direction(_: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> Direction {
                let Some(intersection) = corner
                    .intersection
                    .as_ref()
                    .and_then(|intersection| intersection.event)
                else {
                    return Direction::Forward;
                };

                match intersection {
                    Event::Entry => Direction::Forward,
                    Event::Exit => Direction::Backward,
                }
            }
        }

        Clipper::default()
            .with_operator::<AndOperator<T>>()
            .with_tolerance(tolerance)
            .with_subject(self)
            .with_clip(other)
            .execute()
    }
}

impl<T> Shape<T>
where
    T: Geometry,
    T::Vertex: Vertex,
{
    /// Returns the amount of times this shape winds around the given [`Vertex`].
    fn winding(&self, vertex: &T::Vertex, tolerance: &<T::Vertex as IsClose>::Tolerance) -> isize {
        self.boundaries
            .iter()
            .map(|boundary| boundary.winding(vertex, tolerance))
            .sum()
    }

    /// Returns true if, and only if, the given [`Vertex`] lies inside this shape.
    pub(crate) fn contains(
        &self,
        vertex: &T::Vertex,
        tolerance: &<T::Vertex as IsClose>::Tolerance,
    ) -> bool {
        self.winding(vertex, tolerance) != 0
    }
}

impl<T> Shape<T>
where
    T: Geometry,
{
    /// Creates a new shape from the given boundary.
    pub fn new(value: impl Into<T>) -> Self {
        let boundary = value.into();

        Self {
            boundaries: vec![if boundary.is_clockwise() {
                boundary.reversed()
            } else {
                boundary
            }],
        }
    }

    /// Returns the amount of vertices in this shape.
    pub(crate) fn total_vertices(&self) -> usize {
        self.boundaries
            .iter()
            .map(|boundary| boundary.total_vertices())
            .sum()
    }

    pub(crate) fn edges(&self) -> impl Iterator<Item = T::Edge<'_>> {
        self.boundaries.iter().flat_map(|boundary| boundary.edges())
    }
}

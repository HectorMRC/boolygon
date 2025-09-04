use crate::{Geometry, IsClose, Shape};

use super::Clipper;

/// The operands in a clipping operation.
pub struct Operands<'a, T> {
    /// The shape being clipped.
    pub subject: &'a Shape<T>,
    /// The shape clipping the subject.
    pub clip: &'a Shape<T>,
}

impl<T> Copy for Operands<'_, T> where T: Geometry {}
impl<T> Clone for Operands<'_, T>
where
    T: Geometry,
{
    fn clone(&self) -> Self {
        Self {
            subject: self.subject,
            clip: self.clip,
        }
    }
}

/// The context of a clipping operation.
pub struct Context<'a, T>
where
    T: Geometry,
{
    /// The original operands in this operation.
    pub operands: Operands<'a, T>,
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
            operands: self.operands,
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
            operands: Operands{
                subject: &clipper.subject,
                clip: &clipper.clip,
            },
            tolerance: &clipper.tolerance,
        }
    }
}
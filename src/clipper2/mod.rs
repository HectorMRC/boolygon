mod context;

use std::marker::PhantomData;

use crate::{Corner, Geometry, IsClose, Shape, direction::Direction};

pub use self::context::{Context, Operands};

/// The operation to perform by the clipping algorithm.
pub(crate) trait Operator<T>
where
    T: Geometry,
{
    /// Returns true if, and only if, the given node belongs to the output of the clipping
    /// operation.
    fn is_output(ctx: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> bool;

    /// Returns the direction to take from the given node.
    fn direction(ctx: Context<'_, T>, corner: Corner<'_, T::Vertex>) -> Option<Direction>;
}

/// Implements the clipping algorithm.                                                                                                                                    
pub(crate) struct Clipper<'a, Subject, Clip, Operator, Tolerance> {
    subject: &'a Subject,
    clip: &'a Clip,
    tolerance: Tolerance,
    operator: PhantomData<Operator>,
}

impl<'a, T, Op, Tol> Clipper<'a, Shape<T>, Shape<T>, Op, Tol>
where
    T: Geometry,
    T::Vertex: IsClose<Tolerance = Tol>,
{
    /// Returns the context of this clipping operation.
    pub(self) fn context(&self) -> Context<'_, T> {
        Context {
            operands: Operands {
                subject: &self.subject,
                clip: &self.clip,
            },
            tolerance: &self.tolerance,
        }
    }
}

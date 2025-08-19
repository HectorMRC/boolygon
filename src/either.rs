/// The intersection between two edges.
#[derive(Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Return true if, and only if, is right.
    pub(crate) fn is_right(&self) -> bool {
        matches!(self, Self::Right(_))
    }
}

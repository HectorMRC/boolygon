use num_traits::{Float, Signed};

/// A value that is always positive.
#[derive(Debug, Default, Clone, Copy)]
pub struct Positive<T>(T);

impl<T> From<T> for Positive<T>
where
    T: Signed,
{
    fn from(value: T) -> Self {
        Self(value.abs())
    }
}

impl<T> Positive<T> {
    /// Returns the inner value of self.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// The acceptable deviation between two values.
#[derive(Debug, Default, Clone, Copy)]
pub struct Tolerance<T> {
    /// The maximum allowed difference between two values.
    pub relative: Positive<T>,
    /// Used to compare values near zero.
    pub absolute: Positive<T>,
}

/// A value whose equality depends on a tolerance.
pub trait IsClose {
    type Scalar;

    /// Returns true if, and only if, self and rhs are close enough given a tolerance;
    /// otherwise returns false.
    fn is_close(&self, rhs: &Self, tolerance: &Tolerance<Self::Scalar>) -> bool;
}

impl<T> IsClose for T
where
    T: Float,
{
    type Scalar = T;

    fn is_close(&self, rhs: &Self, tolerance: &Tolerance<Self::Scalar>) -> bool {
        (*self - *rhs).abs()
            <= Self::max(
                tolerance.relative.0 * Self::max(self.abs(), rhs.abs()),
                tolerance.absolute.0,
            )
    }
}

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
    type Tolerance;

    /// Returns true if, and only if, this and the other are close enough given a
    /// tolerance; otherwise returns false.
    fn is_close(&self, other: &Self, tolerance: &Self::Tolerance) -> bool;
}

impl<T> IsClose for T
where
    T: Float,
{
    type Tolerance = Tolerance<T>;

    fn is_close(&self, other: &Self, tolerance: &Self::Tolerance) -> bool {
        (*self - *other).abs()
            <= Self::max(
                tolerance.relative.0 * Self::max(self.abs(), other.abs()),
                tolerance.absolute.0,
            )
    }
}

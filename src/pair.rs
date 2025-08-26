/// A type that may contain a single value or a pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaybePair<T> {
    Single(T),
    Pair([T; 2]),
}

impl<T> MaybePair<T> 
where T: PartialEq {
    /// Returns true if, and only if, the given value is in this [`MaybePair`].
    pub(crate) fn contains(&self, other: &T) -> bool {
        match self {
            MaybePair::Single(value) => value == other,
            MaybePair::Pair(values) => values.contains(other),
        }
    }
}
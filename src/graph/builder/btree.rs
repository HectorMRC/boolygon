use std::{cmp::Ordering, collections::BTreeMap};

/// The key for the [`PartialOrdBTreeMap`].
#[derive(Debug, PartialEq)]
pub struct PartialOrdKey<T>(T);

impl<T> PartialOrd for PartialOrdKey<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for PartialOrdKey<T> where T: PartialEq {}
impl<T> Ord for PartialOrdKey<T>
where
    T: PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Less)
    }
}

impl<T> From<T> for PartialOrdKey<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// A [`BTreeMap`] that accepts keys that only implements [`PartialOrd`].
#[derive(Debug)]
pub struct PartialOrdBTreeMap<K, V>(BTreeMap<PartialOrdKey<K>, V>);

impl<K, V> PartialOrdBTreeMap<K, V>
where
    K: PartialOrd,
{
    pub fn insert(&mut self, key: K, value: V) {
        self.0.insert(key.into(), value);
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.0.get(&key.into())
    }
}

impl<K, V> PartialOrdBTreeMap<K, V> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

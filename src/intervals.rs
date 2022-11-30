use std::collections::btree_map::BTreeMap;
use thiserror::Error;

/// Non-overlapping interval tree based on [BTreeMap]
#[derive(Clone, Debug)]
pub struct IntervalBTreeMap<K, V, S = K>(BTreeMap<K, (S, V)>);

impl<K, V, S> IntervalBTreeMap<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let (&slf_key, (size, value)) = self.0.range(..=key).next_back()?;
        // slf_key <= key < slf_key + size
        // range guaranties that slf_key <= key
        if key < slf_key + *size {
            Some(value)
        } else {
            None
        }
    }

    pub fn contains(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    pub fn get_key_size(&self, key: K) -> Option<(K, S)> {
        let (&slf_key, (size, _value)) = self.0.range(..=key).next_back()?;
        // range guaranties that slf_key <= key
        if key < slf_key + *size {
            Some((slf_key, *size))
        } else {
            None
        }
    }

    /// Left-most overlapping interval
    pub fn overlapped_by(&self, key: K, size: S) -> Option<(K, S)> {
        // Some interval's right end is inside the given one
        if let Some(k_s) = self.get_key_size(key) {
            return Some(k_s);
        }
        // Some interval's left end is inside the given one
        self.0
            .range(key..key + size)
            .next()
            .map(|(k, (s, _v))| (*k, *s))
    }

    /// Return true if tree overlaps with a given interval
    pub fn overlaps(&self, key: K, size: S) -> bool {
        self.overlapped_by(key, size).is_some()
    }

    pub fn try_insert(&mut self, key: K, size: S, value: V) -> Result<(), OccupiedError<K, S>> {
        if let Some((slf_key, slf_size)) = self.overlapped_by(key, size) {
            return Err(OccupiedError {
                key: slf_key,
                size: slf_size,
            });
        }
        // Is it a logical error if it returns something
        match self.0.insert(key, (size, value)) {
            Some(_) => panic!("Interval {:?}..{:?} is already occupied", key, key + size),
            None => Ok(()),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V, S> Default for IntervalBTreeMap<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V, S> From<IntervalBTreeMap<K, V, S>> for BTreeMap<K, (S, V)> {
    fn from(value: IntervalBTreeMap<K, V, S>) -> Self {
        value.0
    }
}

/// Non-overlapping interval tree based on two [Vec]s
#[derive(Clone, Debug)]
pub struct IntervalVec<K, V, S = K> {
    keys: Vec<K>,
    sizes_values: Vec<(S, V)>,
}

impl<K, V, S> IntervalVec<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    pub fn get(&self, key: K) -> Option<&V> {
        match self.keys.binary_search(&key) {
            Ok(index) => Some(&self.sizes_values[index].1),
            Err(0) => None,
            Err(mut index) => {
                index -= 1;
                let (size, value) = &self.sizes_values[index];
                if key < self.keys[index] + *size {
                    Some(value)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    pub fn get_key_size(&self, key: K) -> Option<(K, S)> {
        match self.keys.binary_search(&key) {
            Ok(index) => Some((self.keys[index], self.sizes_values[index].0)),
            Err(0) => None,
            Err(mut index) => {
                index -= 1;
                let slf_key = self.keys[index];
                let (size, _value) = &self.sizes_values[index];
                if key < slf_key + *size {
                    Some((slf_key, *size))
                } else {
                    None
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl<K, V, S> From<IntervalBTreeMap<K, V, S>> for IntervalVec<K, V, S> {
    fn from(btree_map: IntervalBTreeMap<K, V, S>) -> Self {
        let (keys, sizes_values) = btree_map.0.into_iter().unzip();
        Self { keys, sizes_values }
    }
}

impl<K, V, S> From<IntervalVec<K, V, S>> for IntervalBTreeMap<K, V, S>
where
    K: Ord,
{
    fn from(vecs: IntervalVec<K, V, S>) -> Self {
        Self(
            vecs.keys
                .into_iter()
                .zip(vecs.sizes_values.into_iter())
                .collect(),
        )
    }
}

/// Non-overlapping interval collection
#[derive(Clone, Debug)]
pub enum Intervals<K, V, S = K> {
    Ro(IntervalVec<K, V, S>),
    Rw(IntervalBTreeMap<K, V, S>),
}

impl<K, V, S> Intervals<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    pub fn new() -> Self {
        Self::Rw(IntervalBTreeMap::new())
    }

    pub fn get(&self, key: K) -> Option<&V> {
        match self {
            Self::Ro(x) => x.get(key),
            Self::Rw(x) => x.get(key),
        }
    }

    pub fn contains(&self, key: K) -> bool {
        match self {
            Self::Ro(x) => x.contains(key),
            Self::Rw(x) => x.contains(key),
        }
    }

    pub fn get_key_size(&self, key: K) -> Option<(K, S)> {
        match self {
            Self::Ro(x) => x.get_key_size(key),
            Self::Rw(x) => x.get_key_size(key),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Ro(x) => x.len(),
            Self::Rw(x) => x.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Ro(x) => x.is_empty(),
            Self::Rw(x) => x.is_empty(),
        }
    }

    pub fn into_interval_vec(self) -> IntervalVec<K, V, S> {
        match self {
            Self::Ro(x) => x,
            Self::Rw(x) => x.into(),
        }
    }

    pub fn into_interval_btree_map(self) -> IntervalBTreeMap<K, V, S> {
        match self {
            Self::Ro(x) => x.into(),
            Self::Rw(x) => x,
        }
    }

    pub fn into_ro(self) -> Self {
        match self {
            Self::Ro(x) => Self::Ro(x),
            Self::Rw(x) => Self::Ro(x.into()),
        }
    }

    pub fn into_rw(self) -> Self {
        match self {
            Self::Ro(x) => Self::Rw(x.into()),
            Self::Rw(x) => Self::Rw(x),
        }
    }
}

impl<K, V, S> Default for Intervals<K, V, S>
where
    K: std::ops::Add<S, Output = K> + Ord,
    K: Copy + std::fmt::Debug,
    S: Copy,
{
    fn default() -> Self {
        Self::Rw(IntervalBTreeMap::default())
    }
}

impl<K, V, S> From<IntervalVec<K, V, S>> for Intervals<K, V, S> {
    fn from(value: IntervalVec<K, V, S>) -> Self {
        Self::Ro(value)
    }
}

impl<K, V, S> From<IntervalBTreeMap<K, V, S>> for Intervals<K, V, S> {
    fn from(value: IntervalBTreeMap<K, V, S>) -> Self {
        Self::Rw(value)
    }
}

#[derive(Debug, Error)]
pub struct OccupiedError<K, S> {
    pub key: K,
    pub size: S,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear() {
        let mut interval_tree = IntervalBTreeMap::new();
        assert_eq!(interval_tree.len(), 0);
        assert!(interval_tree.is_empty());
        interval_tree.try_insert(0, 1, 0).unwrap();
        assert_eq!(interval_tree.len(), 1);
        assert!(!interval_tree.is_empty());
        interval_tree.clear();
        assert_eq!(interval_tree.len(), 0);
        assert!(interval_tree.is_empty());
    }

    #[test]
    fn no_overlap() {
        let mut interval_tree = IntervalBTreeMap::new();
        interval_tree.try_insert(0, 2, 0).unwrap();
        interval_tree.try_insert(2, 2, 1).unwrap();
        interval_tree.try_insert(4, 2, 2).unwrap();

        assert_eq!(interval_tree.get(0), Some(&0));
        assert_eq!(interval_tree.get(1), Some(&0));
        assert_eq!(interval_tree.get(2), Some(&1));
        assert_eq!(interval_tree.get(3), Some(&1));
        assert_eq!(interval_tree.get(4), Some(&2));
        assert_eq!(interval_tree.get(5), Some(&2));
        assert_eq!(interval_tree.get(-1), None);
        assert_eq!(interval_tree.get(6), None);
        assert_eq!(interval_tree.get(100), None);

        assert_eq!(interval_tree.get_key_size(0), Some((0, 2)));
        assert_eq!(interval_tree.get_key_size(1), Some((0, 2)));
        assert_eq!(interval_tree.get_key_size(2), Some((2, 2)));
        assert_eq!(interval_tree.get_key_size(3), Some((2, 2)));
        assert_eq!(interval_tree.get_key_size(4), Some((4, 2)));
        assert_eq!(interval_tree.get_key_size(5), Some((4, 2)));
        assert_eq!(interval_tree.get_key_size(-1), None);
        assert_eq!(interval_tree.get_key_size(6), None);
        assert_eq!(interval_tree.get_key_size(100), None);
    }

    #[test]
    fn overlaps() {
        let mut interval_tree = IntervalBTreeMap::new();
        interval_tree.try_insert(0, 2, 0).unwrap();
        interval_tree.try_insert(4, 2, 2).unwrap();
        assert!(interval_tree.overlaps(0, 1));
        assert!(interval_tree.overlaps(1, 1));
        assert!(!interval_tree.overlaps(2, 1));
        assert!(interval_tree.overlaps(5, 1));
        assert!(!interval_tree.overlaps(6, 1));
        assert!(!interval_tree.overlaps(7, 1));
        assert!(interval_tree.overlaps(1, 2));
        assert!(interval_tree.overlaps(4, 2));
        assert!(interval_tree.overlaps(4, 100));
        assert!(interval_tree.overlaps(0, 6));
    }
}

#![allow(dead_code)]
use std::collections::btree_map::BTreeMap;
use thiserror::Error;

/// Non-overlapping interval tree based on [BTreeMap]
pub struct IntervalTreeMap<K, V, S = K>(BTreeMap<K, (S, V)>);

impl<K, V, S> IntervalTreeMap<K, V, S>
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

#[derive(Debug, Error)]
pub struct OccupiedError<K, S> {
    pub key: K,
    pub size: S,
}

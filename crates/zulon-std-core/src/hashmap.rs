// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hash map implementation (HashMap<K, V>)

use crate::traits::{Clone, PartialEq};
use crate::Vec;
use crate::Optional;

/// A hash map based on chaining with Vec buckets
/// Simplified implementation for educational purposes
#[derive(Debug)]
pub struct HashMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            entries: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        HashMap {
            entries: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn insert(&mut self, key: K, value: V)
    where
        K: PartialEq + Clone,
        V: Clone,
    {
        // Check if key already exists
        for i in 0..self.entries.len() {
            if self.entries.as_slice()[i].0.eq(&key) {
                // Update existing entry
                self.entries.as_mut_slice()[i] = (key, value);
                return;
            }
        }

        // Insert new entry
        self.entries.push((key, value));
    }

    pub fn get(&self, key: &K) -> Optional<&V>
    where
        K: PartialEq,
    {
        for i in 0..self.entries.len() {
            let entry = &self.entries.as_slice()[i];
            if entry.0.eq(key) {
                return Optional::Some(&entry.1);
            }
        }

        Optional::None
    }

    pub fn get_mut(&mut self, key: &K) -> Optional<&mut V>
    where
        K: PartialEq,
    {
        for i in 0..self.entries.len() {
            let entry = &self.entries.as_slice()[i];
            if entry.0.eq(key) {
                return Optional::Some(&mut self.entries.as_mut_slice()[i].1);
            }
        }

        Optional::None
    }

    pub fn remove(&mut self, key: &K) -> Optional<V>
    where
        K: PartialEq,
    {
        for i in 0..self.entries.len() {
            if self.entries.as_slice()[i].0.eq(key) {
                let entry = self.entries.remove(i);
                return Optional::Some(entry.1);
            }
        }

        Optional::None
    }

    pub fn contains_key(&self, key: &K) -> bool
    where
        K: PartialEq,
    {
        match self.get(key) {
            Optional::Some(_) => true,
            Optional::None => false,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            map: self,
            index: 0,
        }
    }
}

impl<K: Clone + PartialEq, V: Clone> Clone for HashMap<K, V> {
    fn clone(&self) -> Self {
        HashMap {
            entries: self.entries.clone(),
        }
    }
}

impl<K, V> Drop for HashMap<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    index: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn next(&mut self) -> Optional<(&'a K, &'a V)> {
        if self.index < self.map.len() {
            let entry = &self.map.entries.as_slice()[self.index];
            self.index += 1;
            Optional::Some((&entry.0, &entry.1))
        } else {
            Optional::None
        }
    }
}

/// Hash trait for types that can be hashed
/// Note: In this simplified version, we use linear search instead of hashing
pub trait Hash {
    fn hash(&self) -> usize;
}

// Implement Hash for primitive types
impl Hash for i32 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl Hash for i64 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl Hash for u32 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl Hash for u64 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl Hash for usize {
    fn hash(&self) -> usize {
        *self
    }
}

impl Hash for bool {
    fn hash(&self) -> usize {
        if *self { 1 } else { 0 }
    }
}

impl Hash for char {
    fn hash(&self) -> usize {
        *self as usize
    }
}

// Implement Hash for &str using FNV-1a
impl Hash for &str {
    fn hash(&self) -> usize {
        const FNV_PRIME: usize = 1099511628211;
        const FNV_OFFSET: usize = 14695981039346656037;

        let mut hash = FNV_OFFSET;
        for byte in self.bytes() {
            hash ^= byte as usize;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let map: HashMap<i32, i32> = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_insert_and_get() {
        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);
        map.insert(3, 30);

        assert_eq!(map.len(), 3);

        match map.get(&2) {
            Optional::Some(val) => assert_eq!(*val, 20),
            Optional::None => panic!("Expected Some(20)"),
        }
    }

    #[test]
    fn test_get_none() {
        let map: HashMap<i32, i32> = HashMap::new();
        match map.get(&1) {
            Optional::Some(_) => panic!("Expected None"),
            Optional::None => {},
        }
    }

    #[test]
    fn test_contains_key() {
        let mut map = HashMap::new();
        map.insert(1, 10);

        assert!(map.contains_key(&1));
        assert!(!map.contains_key(&2));
    }

    #[test]
    fn test_remove() {
        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);

        let removed = map.remove(&1);
        match removed {
            Optional::Some(val) => assert_eq!(val, 10),
            Optional::None => panic!("Expected Some(10)"),
        }

        assert_eq!(map.len(), 1);
        assert!(!map.contains_key(&1));
    }

    #[test]
    fn test_update_value() {
        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(1, 20);

        match map.get(&1) {
            Optional::Some(val) => assert_eq!(*val, 20),
            Optional::None => panic!("Expected Some(20)"),
        }

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);
        map.clear();

        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_clone() {
        let mut map = HashMap::new();
        map.insert(1, 10);
        map.insert(2, 20);

        let map2 = map.clone();
        assert_eq!(map2.len(), map.len());
        assert!(map2.contains_key(&1));
        assert!(map2.contains_key(&2));
    }
}

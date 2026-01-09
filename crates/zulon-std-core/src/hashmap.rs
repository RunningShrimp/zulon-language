// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hash map implementation (HashMap<K, V>)

use crate::traits::{Clone, PartialEq, Hash};
use crate::Vec;
use crate::Optional;

/// Default initial capacity for HashMap
const DEFAULT_CAPACITY: usize = 16;

/// Load factor threshold for resizing (3/4 = 75%)
const LOAD_FACTOR_NUMERATOR: usize = 3;
const LOAD_FACTOR_DENOMINATOR: usize = 4;

/// A hash map based on chaining with Vec buckets
/// O(1) average case for insert, get, and remove operations
#[derive(Debug)]
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    capacity: usize,
    length: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        let mut buckets = Vec::new();
        for _ in 0..DEFAULT_CAPACITY {
            buckets.push(Vec::new());
        }

        HashMap {
            buckets,
            capacity: DEFAULT_CAPACITY,
            length: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(1);
        let mut buckets = Vec::new();
        for _ in 0..cap {
            buckets.push(Vec::new());
        }

        HashMap {
            buckets,
            capacity: cap,
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Compute bucket index for a key
    fn hash_key(&self, key: &K) -> usize
    where
        K: Hash,
    {
        let hash = key.hash();
        (hash as usize) % self.capacity
    }

    /// Resize the HashMap when load factor is exceeded
    fn resize(&mut self, new_capacity: usize)
    where
        K: Hash + Clone + PartialEq,
        V: Clone,
    {
        let old_buckets = std::mem::replace(&mut self.buckets, Vec::new());

        // Create new buckets
        let mut new_buckets = Vec::new();
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }

        self.capacity = new_capacity;
        self.buckets = new_buckets;
        self.length = 0;

        // Rehash all entries into new buckets
        let old_slice = old_buckets.as_slice();
        for i in 0..old_slice.len() {
            let bucket = &old_slice[i];
            let bucket_slice = bucket.as_slice();
            for j in 0..bucket_slice.len() {
                let (key, value) = bucket_slice[j].clone();
                self.insert(key, value);
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V)
    where
        K: Hash + PartialEq + Clone,
        V: Clone,
    {
        // Check if we need to resize
        if self.length * LOAD_FACTOR_DENOMINATOR > self.capacity * LOAD_FACTOR_NUMERATOR {
            self.resize(self.capacity * 2);
        }

        let bucket_index = self.hash_key(&key);
        let bucket = &mut self.buckets.as_mut_slice()[bucket_index];

        // Check if key already exists in this bucket
        for i in 0..bucket.len() {
            if bucket.as_slice()[i].0.eq(&key) {
                // Update existing entry
                bucket.as_mut_slice()[i] = (key, value);
                return;
            }
        }

        // Insert new entry
        bucket.push((key, value));
        self.length += 1;
    }

    pub fn get(&self, key: &K) -> Optional<&V>
    where
        K: Hash + PartialEq,
    {
        let bucket_index = self.hash_key(key);
        let bucket = &self.buckets.as_slice()[bucket_index];

        for i in 0..bucket.len() {
            let entry = &bucket.as_slice()[i];
            if entry.0.eq(key) {
                return Optional::Some(&entry.1);
            }
        }

        Optional::None
    }

    pub fn get_mut(&mut self, key: &K) -> Optional<&mut V>
    where
        K: Hash + PartialEq,
    {
        let bucket_index = self.hash_key(key);
        let bucket = &mut self.buckets.as_mut_slice()[bucket_index];

        for i in 0..bucket.len() {
            if bucket.as_slice()[i].0.eq(key) {
                return Optional::Some(&mut bucket.as_mut_slice()[i].1);
            }
        }

        Optional::None
    }

    pub fn remove(&mut self, key: &K) -> Optional<V>
    where
        K: Hash + PartialEq,
    {
        let bucket_index = self.hash_key(key);
        let bucket = &mut self.buckets.as_mut_slice()[bucket_index];

        for i in 0..bucket.len() {
            if bucket.as_slice()[i].0.eq(key) {
                let entry = bucket.remove(i);
                self.length -= 1;
                return Optional::Some(entry.1);
            }
        }

        Optional::None
    }

    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Hash + PartialEq,
    {
        match self.get(key) {
            Optional::Some(_) => true,
            Optional::None => false,
        }
    }

    pub fn clear(&mut self) {
        let buckets = &mut self.buckets;
        for i in 0..buckets.as_slice().len() {
            buckets.as_mut_slice()[i].clear();
        }
        self.length = 0;
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            map: self,
            bucket_index: 0,
            entry_index: 0,
        }
    }
}

impl<K: Clone, V: Clone> Clone for HashMap<K, V> {
    fn clone(&self) -> Self {
        HashMap {
            buckets: self.buckets.clone(),
            capacity: self.capacity,
            length: self.length,
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
    bucket_index: usize,
    entry_index: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    pub fn next(&mut self) -> Optional<(&'a K, &'a V)> {
        // Find the next non-empty bucket
        while self.bucket_index < self.map.capacity {
            let buckets_slice = self.map.buckets.as_slice();
            let bucket = &buckets_slice[self.bucket_index];

            if self.entry_index < bucket.len() {
                let entry = &bucket.as_slice()[self.entry_index];
                self.entry_index += 1;
                return Optional::Some((&entry.0, &entry.1));
            }

            // Move to next bucket
            self.bucket_index += 1;
            self.entry_index = 0;
        }

        Optional::None
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

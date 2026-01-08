// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Hash set implementation (HashSet<T>)

use crate::traits::{Clone, PartialEq};
use crate::Vec;
use crate::Optional;

/// A hash set based on a Vec for storage
/// Simplified implementation for educational purposes
#[derive(Debug)]
pub struct HashSet<T> {
    entries: Vec<T>,
}

impl<T> HashSet<T> {
    pub fn new() -> Self {
        HashSet {
            entries: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        HashSet {
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

    pub fn insert(&mut self, value: T)
    where
        T: PartialEq + Clone,
    {
        // Check if value already exists
        for i in 0..self.entries.len() {
            if self.entries.as_slice()[i].eq(&value) {
                // Value already exists, no need to insert
                return;
            }
        }

        // Insert new value
        self.entries.push(value);
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        for i in 0..self.entries.len() {
            if self.entries.as_slice()[i].eq(value) {
                return true;
            }
        }

        false
    }

    pub fn remove(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        for i in 0..self.entries.len() {
            if self.entries.as_slice()[i].eq(value) {
                self.entries.remove(i);
                return true;
            }
        }

        false
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            set: self,
            index: 0,
        }
    }
}

impl<T: Clone + PartialEq> Clone for HashSet<T> {
    fn clone(&self) -> Self {
        HashSet {
            entries: self.entries.clone(),
        }
    }
}

impl<T> Drop for HashSet<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, T> {
    set: &'a HashSet<T>,
    index: usize,
}

impl<'a, T> Iter<'a, T> {
    pub fn next(&mut self) -> Optional<&'a T> {
        if self.index < self.set.len() {
            let value = &self.set.entries.as_slice()[self.index];
            self.index += 1;
            Optional::Some(value)
        } else {
            Optional::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_set() {
        let set: HashSet<i32> = HashSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_insert_and_contains() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        assert_eq!(set.len(), 3);
        assert!(set.contains(&2));
        assert!(!set.contains(&4));
    }

    #[test]
    fn test_no_duplicates() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(1);
        set.insert(1);

        // Duplicates should be ignored
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);

        let removed = set.remove(&2);
        assert!(removed);
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&2));

        let removed_again = set.remove(&2);
        assert!(!removed_again);
    }

    #[test]
    fn test_clear() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.clear();

        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_clone() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);

        let set2 = set.clone();
        assert_eq!(set2.len(), set.len());
        assert!(set2.contains(&1));
        assert!(set2.contains(&2));
    }

    #[test]
    fn test_with_capacity() {
        let set: HashSet<i32> = HashSet::with_capacity(10);
        assert_eq!(set.len(), 0);
        assert!(set.capacity() >= 10);
    }
}

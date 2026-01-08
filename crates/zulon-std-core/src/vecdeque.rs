// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Double-ended queue (VecDeque<T>) implementation

use crate::traits::{Clone, PartialEq};
use crate::Vec;
use crate::Optional;

/// A double-ended queue implemented with a ring buffer
/// Simplified implementation using Vec for storage
#[derive(Debug)]
pub struct VecDeque<T> {
    // Simplified: using Vec as backing storage
    // For MVP, we use a simple Vec-based approach
    // TODO: Implement proper ring buffer for O(1) front operations
    data: Vec<T>,
}

impl<T> VecDeque<T> {
    pub fn new() -> Self {
        VecDeque {
            data: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        VecDeque {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Add element to back (O(1) amortized)
    pub fn push_back(&mut self, value: T) {
        self.data.push(value);
    }

    /// Remove element from back (O(1))
    pub fn pop_back(&mut self) -> Optional<T> {
        self.data.pop()
    }

    /// Add element to front (O(n) for MVP)
    pub fn push_front(&mut self, value: T)
    where
        T: Clone,
    {
        // MVP: O(n) operation
        // TODO: Implement ring buffer for O(1)
        let mut new_vec = Vec::with_capacity(self.data.len() + 1);
        new_vec.push(value);

        for i in 0..self.data.len() {
            let elem = &self.data.as_slice()[i];
            new_vec.push(elem.clone());
        }

        self.data = new_vec;
    }

    /// Remove element from front (O(n) for MVP)
    pub fn pop_front(&mut self) -> Optional<T>
    where
        T: Clone,
    {
        // MVP: O(n) operation
        if self.data.is_empty() {
            return Optional::None;
        }

        let front = self.data.remove(0);
        Optional::Some(front)
    }

    /// Get element at front
    pub fn front(&self) -> Optional<&T> {
        if self.data.is_empty() {
            return Optional::None;
        }

        Optional::Some(&self.data.as_slice()[0])
    }

    /// Get element at back
    pub fn back(&self) -> Optional<&T> {
        if self.data.is_empty() {
            return Optional::None;
        }

        let len = self.data.len();
        Optional::Some(&self.data.as_slice()[len - 1])
    }

    /// Get element at index
    pub fn get(&self, index: usize) -> Optional<&T> {
        if index >= self.data.len() {
            return Optional::None;
        }

        Optional::Some(&self.data.as_slice()[index])
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }
}

impl<T: Clone + PartialEq> Clone for VecDeque<T> {
    fn clone(&self) -> Self {
        VecDeque {
            data: self.data.clone(),
        }
    }
}

impl<T> Drop for VecDeque<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_deque() {
        let deque: VecDeque<i32> = VecDeque::new();
        assert_eq!(deque.len(), 0);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_push_back() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        assert_eq!(deque.len(), 3);
    }

    #[test]
    fn test_push_front() {
        let mut deque = VecDeque::new();
        deque.push_front(1);
        deque.push_front(2);
        deque.push_front(3);

        assert_eq!(deque.len(), 3);

        match deque.front() {
            Optional::Some(val) => assert_eq!(*val, 3),
            Optional::None => panic!("Expected Some(3)"),
        }
    }

    #[test]
    fn test_pop_back() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        let popped = deque.pop_back();
        match popped {
            Optional::Some(val) => assert_eq!(val, 3),
            Optional::None => panic!("Expected Some(3)"),
        }

        assert_eq!(deque.len(), 2);
    }

    #[test]
    fn test_pop_front() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        let popped = deque.pop_front();
        match popped {
            Optional::Some(val) => assert_eq!(val, 1),
            Optional::None => panic!("Expected Some(1)"),
        }

        assert_eq!(deque.len(), 2);
    }

    #[test]
    fn test_front_and_back() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        match deque.front() {
            Optional::Some(val) => assert_eq!(*val, 1),
            Optional::None => panic!("Expected Some(1)"),
        }

        match deque.back() {
            Optional::Some(val) => assert_eq!(*val, 3),
            Optional::None => panic!("Expected Some(3)"),
        }
    }

    #[test]
    fn test_get() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.push_back(3);

        match deque.get(1) {
            Optional::Some(val) => assert_eq!(*val, 2),
            Optional::None => panic!("Expected Some(2)"),
        }

        match deque.get(10) {
            Optional::Some(_) => panic!("Expected None"),
            Optional::None => {},
        }
    }

    #[test]
    fn test_clear() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);
        deque.clear();

        assert_eq!(deque.len(), 0);
        assert!(deque.is_empty());
    }

    #[test]
    fn test_clone() {
        let mut deque = VecDeque::new();
        deque.push_back(1);
        deque.push_back(2);

        let deque2 = deque.clone();
        assert_eq!(deque2.len(), deque.len());
    }
}

// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dynamic array (Vec<T>) implementation

use crate::traits::{Clone, PartialEq};
use crate::Optional;
use std::iter::{Iterator, IntoIterator};

/// A growable list type with heap-allocated contents
#[derive(Debug)]
pub struct Vec<T> {
    ptr: *mut T,
    capacity: usize,
    len: usize,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            ptr: std::ptr::null_mut(),
            capacity: 0,
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (ptr, capacity) = if capacity == 0 {
            (std::ptr::null_mut(), 0)
        } else {
            let size = capacity * std::mem::size_of::<T>();
            unsafe {
                (zulon_runtime_alloc(size) as *mut T, capacity)
            }
        };

        Vec { ptr, capacity, len: 0 }
    }

    pub fn len(&self) -> usize { self.len }

    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn capacity(&self) -> usize { self.capacity }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.reserve(1);
        }

        unsafe {
            std::ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Optional<T> {
        if self.len == 0 {
            return Optional::None;
        }

        self.len -= 1;
        unsafe {
            Optional::Some(std::ptr::read(self.ptr.add(self.len)))
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let avail = self.capacity - self.len;
        if avail >= additional {
            return;
        }

        let new_capacity = std::cmp::max(self.capacity * 2, self.len + additional);
        let new_size = new_capacity * std::mem::size_of::<T>();

        unsafe {
            let new_ptr = if new_capacity == 0 {
                std::ptr::null_mut()
            } else {
                zulon_runtime_alloc(new_size) as *mut T
            };

            std::ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);

            if self.capacity > 0 {
                zulon_runtime_free(self.ptr as *const u8);
            }

            self.ptr = new_ptr;
            self.capacity = new_capacity;
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len {
            self.len = new_len;
        }
    }

    /// SAFETY: Caller must ensure that `new_len <= capacity` and that
    /// the elements up to `new_len` are initialized.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("remove index out of bounds");
        }

        self.len -= 1;

        unsafe {
            let element = std::ptr::read(self.ptr.add(index));

            // Shift all elements after index one position to the left
            let count = self.len - index;
            if count > 0 {
                std::ptr::copy(self.ptr.add(index + 1), self.ptr.add(index), count);
            }

            element
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }

    /// Insert an element at a specific index, shifting elements to the right
    pub fn insert(&mut self, index: usize, element: T) {
        if index > self.len {
            panic!("insert index out of bounds");
        }

        // Reserve space if needed
        if self.len == self.capacity {
            self.reserve(1);
        }

        unsafe {
            // Shift elements to the right
            if index < self.len {
                std::ptr::copy(self.ptr.add(index), self.ptr.add(index + 1), self.len - index);
            }

            // Write new element
            std::ptr::write(self.ptr.add(index), element);
        }

        self.len += 1;
    }

    /// Extend the Vec with elements from a slice
    pub fn extend(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.reserve(slice.len());

        for i in 0..slice.len() {
            self.push(slice[i].clone());
        }
    }

    /// Push all elements from another Vec
    pub fn push_all(&mut self, other: &Vec<T>)
    where
        T: Clone,
    {
        self.extend(other.as_slice());
    }

    /// Retain only elements that match the predicate
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut write_idx = 0;

        unsafe {
            for read_idx in 0..self.len {
                let element = &*self.ptr.add(read_idx);

                if f(element) {
                    if read_idx != write_idx {
                        let element = std::ptr::read(self.ptr.add(read_idx));
                        std::ptr::write(self.ptr.add(write_idx), element);
                    }
                    write_idx += 1;
                }
            }

            // Drop remaining elements
            for i in write_idx..self.len {
                std::ptr::drop_in_place(self.ptr.add(i));
            }
        }

        self.len = write_idx;
    }

    /// Remove consecutive duplicate elements
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        if self.len <= 1 {
            return;
        }

        let mut write_idx = 1;

        unsafe {
            for read_idx in 1..self.len {
                let prev = &*self.ptr.add(write_idx - 1);
                let curr = &*self.ptr.add(read_idx);

                if !prev.eq(curr) {
                    if read_idx != write_idx {
                        let element = std::ptr::read(self.ptr.add(read_idx));
                        std::ptr::write(self.ptr.add(write_idx), element);
                    }
                    write_idx += 1;
                } else {
                    // Drop duplicate
                    std::ptr::drop_in_place(self.ptr.add(read_idx));
                }
            }
        }

        self.len = write_idx;
    }

    /// Get the first element, if any
    pub fn first(&self) -> Optional<&T> {
        if self.len == 0 {
            Optional::None
        } else {
            unsafe {
                Optional::Some(&*self.ptr)
            }
        }
    }

    /// Get the last element, if any
    pub fn last(&self) -> Optional<&T> {
        if self.len == 0 {
            Optional::None
        } else {
            unsafe {
                Optional::Some(&*self.ptr.add(self.len - 1))
            }
        }
    }

    /// Get a mutable reference to the first element, if any
    pub fn first_mut(&mut self) -> Optional<&mut T> {
        if self.len == 0 {
            Optional::None
        } else {
            unsafe {
                Optional::Some(&mut *self.ptr)
            }
        }
    }

    /// Get a mutable reference to the last element, if any
    pub fn last_mut(&mut self) -> Optional<&mut T> {
        if self.len == 0 {
            Optional::None
        } else {
            unsafe {
                Optional::Some(&mut *self.ptr.add(self.len - 1))
            }
        }
    }

    /// Reverse the order of elements in-place
    pub fn reverse(&mut self) {
        if self.len <= 1 {
            return;
        }

        let half_len = self.len / 2;

        unsafe {
            for i in 0..half_len {
                let j = self.len - i - 1;

                // Swap elements without dropping
                let a = std::ptr::read(self.ptr.add(i));
                let b = std::ptr::read(self.ptr.add(j));

                std::ptr::write(self.ptr.add(i), b);
                std::ptr::write(self.ptr.add(j), a);
            }
        }
    }
}

// ============================================================================
// Iterator support
// ============================================================================

/// Iterator over Vec<T>
pub struct IntoIter<T> {
    vec: Vec<T>,
    next_idx: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.next_idx < self.vec.len {
            let item = unsafe {
                std::ptr::read(self.vec.ptr.add(self.next_idx))
            };
            self.next_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            vec: self,
            next_idx: 0,
        }
    }
}

/// Iterator over &'a Vec<T>
pub struct Iter<'a, T> {
    vec: &'a Vec<T>,
    next_idx: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.next_idx < self.vec.len {
            let item = unsafe {
                &*self.vec.ptr.add(self.next_idx)
            };
            self.next_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Iterator over &'a mut Vec<T>
pub struct IterMut<'a, T> {
    vec: &'a mut Vec<T>,
    next_idx: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        if self.next_idx < self.vec.len {
            // SAFETY: We use raw pointers and lifetimes to allow mutable iteration
            let item = unsafe {
                &mut *(self.vec.ptr.add(self.next_idx))
            };
            self.next_idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<T> Vec<T> {
    /// Create an iterator over references to the Vec's elements
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            vec: self,
            next_idx: 0,
        }
    }

    /// Create an iterator over mutable references to the Vec's elements
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            vec: self,
            next_idx: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Self {
        let mut new_vec = Vec::with_capacity(self.capacity);

        unsafe {
            for i in 0..self.len {
                let value = std::ptr::read(self.ptr.add(i));
                new_vec.push(value);
            }
        }

        new_vec
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        unsafe {
            for i in 0..self.len {
                let a = std::ptr::read(self.ptr.add(i));
                let b = std::ptr::read(other.ptr.add(i));
                if !a.eq(&b) {
                    return false;
                }
            }
        }

        true
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            if self.capacity > 0 {
                zulon_runtime_free(self.ptr as *const u8);
            }
        }
    }
}

extern "C" {
    fn zulon_runtime_alloc(size: usize) -> *mut u8;
    fn zulon_runtime_free(ptr: *const u8);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vec() {
        let vec: Vec<i32> = Vec::new();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_push_and_pop() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);

        let popped = vec.pop();
        match popped {
            Optional::Some(val) => assert_eq!(val, 3),
            Optional::None => panic!("Expected Some(3)"),
        }

        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_with_capacity() {
        let vec: Vec<i32> = Vec::with_capacity(10);
        assert_eq!(vec.len(), 0);
        assert!(vec.capacity() >= 10);
    }

    #[test]
    fn test_reserve() {
        let mut vec: Vec<i32> = Vec::new();
        vec.reserve(10);
        assert!(vec.capacity() >= 10);
    }

    #[test]
    fn test_clear() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.clear();

        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_truncate() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);

        vec.truncate(3);
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_remove() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let removed = vec.remove(1);
        assert_eq!(removed, 2);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_clone() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let vec2 = vec.clone();
        assert_eq!(vec2.len(), vec.len());

        for i in 0..vec.len() {
            let a = vec.as_slice()[i];
            let b = vec2.as_slice()[i];
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_insert() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(3);
        vec.insert(1, 2);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice()[0], 1);
        assert_eq!(vec.as_slice()[1], 2);
        assert_eq!(vec.as_slice()[2], 3);
    }

    #[test]
    fn test_insert_at_end() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.insert(2, 3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice()[2], 3);
    }

    #[test]
    fn test_extend() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);

        let other = [3, 4, 5];
        vec.extend(&other);

        assert_eq!(vec.len(), 5);
        assert_eq!(vec.as_slice()[0], 1);
        assert_eq!(vec.as_slice()[4], 5);
    }

    #[test]
    fn test_push_all() {
        let mut vec1 = Vec::new();
        vec1.push(1);
        vec1.push(2);

        let mut vec2 = Vec::new();
        vec2.push(3);
        vec2.push(4);

        vec1.push_all(&vec2);

        assert_eq!(vec1.len(), 4);
        assert_eq!(vec1.as_slice()[0], 1);
        assert_eq!(vec1.as_slice()[3], 4);
    }

    #[test]
    fn test_first_last() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        match vec.first() {
            Optional::Some(&val) => assert_eq!(val, 1),
            Optional::None => panic!("Expected Some(1)"),
        }

        match vec.last() {
            Optional::Some(&val) => assert_eq!(val, 3),
            Optional::None => panic!("Expected Some(3)"),
        }
    }

    #[test]
    fn test_first_last_empty() {
        let vec: Vec<i32> = Vec::new();

        match vec.first() {
            Optional::Some(_) => panic!("Expected None"),
            Optional::None => {},
        }

        match vec.last() {
            Optional::Some(_) => panic!("Expected None"),
            Optional::None => {},
        }
    }

    #[test]
    fn test_first_mut_last_mut() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        match vec.first_mut() {
            Optional::Some(val) => *val = 10,
            Optional::None => panic!("Expected Some"),
        }

        match vec.last_mut() {
            Optional::Some(val) => *val = 30,
            Optional::None => panic!("Expected Some"),
        }

        assert_eq!(vec.as_slice()[0], 10);
        assert_eq!(vec.as_slice()[2], 30);
    }

    #[test]
    fn test_retain() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);

        vec.retain(|&x| x % 2 == 0);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.as_slice()[0], 2);
        assert_eq!(vec.as_slice()[1], 4);
    }

    #[test]
    fn test_dedup() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(1);
        vec.push(2);
        vec.push(2);
        vec.push(2);
        vec.push(3);

        vec.dedup();

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice()[0], 1);
        assert_eq!(vec.as_slice()[1], 2);
        assert_eq!(vec.as_slice()[2], 3);
    }

    #[test]
    fn test_reverse() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
        vec.push(5);

        vec.reverse();

        assert_eq!(vec.len(), 5);
        assert_eq!(vec.as_slice()[0], 5);
        assert_eq!(vec.as_slice()[4], 1);
    }

    #[test]
    fn test_reverse_single() {
        let mut vec = Vec::new();
        vec.push(42);

        vec.reverse();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.as_slice()[0], 42);
    }

    #[test]
    fn test_iter() {
        let vec = create_test_vec();
        let mut iter = vec.iter();

        match iter.next() {
            Some(&val) => assert_eq!(val, 1),
            None => panic!("Expected Some(1)"),
        }

        match iter.next() {
            Some(&val) => assert_eq!(val, 2),
            None => panic!("Expected Some(2)"),
        }

        match iter.next() {
            Some(&val) => assert_eq!(val, 3),
            None => panic!("Expected Some(3)"),
        }

        match iter.next() {
            Some(_) => panic!("Expected None"),
            None => {},
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        {
            let mut iter = vec.iter_mut();
            while let Some(val) = iter.next() {
                *val *= 2;
            }
        }

        assert_eq!(vec.as_slice()[0], 2);
        assert_eq!(vec.as_slice()[1], 4);
        assert_eq!(vec.as_slice()[2], 6);
    }

    #[test]
    fn test_into_iter() {
        let vec = create_test_vec();
        let mut iter = vec.into_iter();

        match iter.next() {
            Some(val) => assert_eq!(val, 1),
            None => panic!("Expected Some(1)"),
        }

        match iter.next() {
            Some(val) => assert_eq!(val, 2),
            None => panic!("Expected Some(2)"),
        }

        match iter.next() {
            Some(val) => assert_eq!(val, 3),
            None => panic!("Expected Some(3)"),
        }

        match iter.next() {
            Some(_) => panic!("Expected None"),
            None => {},
        }
    }

    #[test]
    fn test_iterator_count() {
        let vec = create_test_vec();
        let iter = vec.iter();
        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn test_iterator_last() {
        let vec = create_test_vec();
        let iter = vec.iter();

        match iter.last() {
            Some(&val) => assert_eq!(val, 3),
            None => panic!("Expected Some(3)"),
        }
    }

    #[test]
    fn test_iterator_nth() {
        let vec = create_test_vec();
        let mut iter = vec.iter();

        match iter.nth(1) {
            Some(&val) => assert_eq!(val, 2),
            None => panic!("Expected Some(2)"),
        }

        match iter.next() {
            Some(&val) => assert_eq!(val, 3),
            None => panic!("Expected Some(3)"),
        }
    }

    fn create_test_vec() -> Vec<i32> {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec
    }
}

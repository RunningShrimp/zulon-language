// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dynamic array (Vec<T>) implementation

use crate::traits::{Clone, PartialEq};
use crate::Optional;

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
}

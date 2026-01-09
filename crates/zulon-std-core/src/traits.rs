// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Core traits for ZULON standard library
//!
//! These traits provide foundational functionality used throughout the standard library.

/// Equality comparison
///
/// `PartialEq` is a partial equivalence relation, meaning values can be compared
/// for equality but the relation may not be reflexive (e.g., NaN != NaN).
pub trait PartialEq<Rhs: ?Sized = Self> {
    /// Test whether `self` and `other` are equal
    fn eq(&self, other: &Rhs) -> bool;

    /// Test whether `self` and `other` are not equal
    fn ne(&self, other: &Rhs) -> bool {
        !self.eq(other)
    }
}

/// Total equality comparison
///
/// `Eq` is a full equivalence relation, meaning the comparison is reflexive
/// (every value is equal to itself).
///
/// This trait has no methods, but serves as a marker trait.
pub trait Eq: PartialEq<Self> {
    // This trait is a marker with no methods
}

/// Partial ordering comparison
///
/// `PartialOrd` is a partial ordering relation, meaning some values may not be
/// comparable (e.g., NaN < NaN is false, NaN >= NaN is also false).
pub trait PartialOrd: PartialEq<Self> {
    /// Test whether `self` is less than `other`
    fn lt(&self, other: &Self) -> bool;

    /// Test whether `self` is less than or equal to `other`
    fn le(&self, other: &Self) -> bool {
        self.lt(other) || self.eq(other)
    }

    /// Test whether `self` is greater than `other`
    fn gt(&self, other: &Self) -> bool {
        other.lt(self)
    }

    /// Test whether `self` is greater than or equal to `other`
    fn ge(&self, other: &Self) -> bool {
        other.le(self)
    }

    /// Compare two values, returning `Less`, `Equal`, or `Greater`
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}

/// Total ordering comparison
///
/// `Ord` is a total ordering relation, meaning all values are comparable.
pub trait Ord: Eq + PartialOrd {
    /// Compare two values, returning `Less`, `Equal`, or `Greater`
    fn cmp(&self, other: &Self) -> Ordering;
}

/// Ordering result from comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    /// Less than
    Less,
    /// Equal
    Equal,
    /// Greater than
    Greater,
}

/// Hash trait
///
/// Types that implement `Hash` can be hashed into a u64 value.
/// This is used for HashMap keys and other hash-based data structures.
pub trait Hash {
    /// Compute a hash value for `self`
    fn hash(&self) -> u64;
}

/// Copy trait
///
/// Types that implement `Copy` are copied bitwise when assigned.
/// This trait is a marker with no methods.
///
/// # Examples
/// - All primitive numeric types implement `Copy`
/// - `bool` implements `Copy`
/// - References (`&T`) implement `Copy`
pub trait Copy: Clone {
    // Empty marker trait
}

/// Clone trait
///
/// Types that can create a deep copy of themselves.
pub trait Clone {
    /// Create a deep copy of `self`
    fn clone(&self) -> Self;

    /// Clone `self` and convert into the destination type
    fn clone_into(&self, dest: &mut Self)
    where
        Self: Sized,
    {
        *dest = self.clone();
    }
}

// ============================================================================
// Implementations for primitive types
// ============================================================================

macro_rules! impl_eq {
    ($($t:ty),*) => {
        $(
            impl PartialEq for $t {
                #[inline]
                fn eq(&self, other: &Self) -> bool {
                    *self == *other
                }
            }

            impl Eq for $t {}
        )*
    };
}

// Implement PartialEq and Eq for primitive types
impl_eq!(i8, i16, i32, i64, i128, isize);
impl_eq!(u8, u16, u32, u64, u128, usize);
impl_eq!(bool, char);

// Implement for f32 and f32 (Note: NaN != NaN, so only PartialOrd, not Eq)
impl PartialEq for f32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl PartialEq for f64 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        *self == *other
    }
}

// Implement Copy for primitive types
macro_rules! impl_copy {
    ($($t:ty),*) => {
        $(
            impl Clone for $t {
                #[inline]
                fn clone(&self) -> Self {
                    *self
                }
            }

            impl Copy for $t {}
        )*
    };
}

impl_copy!(i8, i16, i32, i64, i128, isize);
impl_copy!(u8, u16, u32, u64, u128, usize);
impl_copy!(f32, f64);
impl_copy!(bool, char);

// Implement PartialOrd and Ord for signed integers
macro_rules! impl_partial_ord_signed {
    ($($t:ty),*) => {
        $(
            impl PartialOrd for $t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    if *self < *other {
                        Some(Ordering::Less)
                    } else if *self > *other {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Equal)
                    }
                }

                #[inline]
                fn lt(&self, other: &Self) -> bool {
                    *self < *other
                }

                #[inline]
                fn le(&self, other: &Self) -> bool {
                    *self <= *other
                }

                #[inline]
                fn gt(&self, other: &Self) -> bool {
                    *self > *other
                }

                #[inline]
                fn ge(&self, other: &Self) -> bool {
                    *self >= *other
                }
            }

            impl Ord for $t {
                #[inline]
                fn cmp(&self, other: &Self) -> Ordering {
                    if *self < *other {
                        Ordering::Less
                    } else if *self > *other {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
            }
        )*
    };
}

impl_partial_ord_signed!(i8, i16, i32, i64, i128, isize);

// Implement PartialOrd and Ord for unsigned integers
macro_rules! impl_partial_ord_unsigned {
    ($($t:ty),*) => {
        $(
            impl PartialOrd for $t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    if *self < *other {
                        Some(Ordering::Less)
                    } else if *self > *other {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Equal)
                    }
                }

                #[inline]
                fn lt(&self, other: &Self) -> bool {
                    *self < *other
                }

                #[inline]
                fn le(&self, other: &Self) -> bool {
                    *self <= *other
                }

                #[inline]
                fn gt(&self, other: &Self) -> bool {
                    *self > *other
                }

                #[inline]
                fn ge(&self, other: &Self) -> bool {
                    *self >= *other
                }
            }

            impl Ord for $t {
                #[inline]
                fn cmp(&self, other: &Self) -> Ordering {
                    if *self < *other {
                        Ordering::Less
                    } else if *self > *other {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    }
                }
            }
        )*
    };
}

impl_partial_ord_unsigned!(u8, u16, u32, u64, u128, usize);

// Implement PartialOrd for bool
impl PartialOrd for bool {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if *self < *other {
            Some(Ordering::Less)
        } else if *self > *other {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }
}

impl Ord for bool {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if *self < *other {
            Ordering::Less
        } else if *self > *other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

// Implement PartialOrd for char (based on Unicode code point)
impl PartialOrd for char {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if *self < *other {
            Some(Ordering::Less)
        } else if *self > *other {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        (*self as u32) < (*other as u32)
    }
}

impl Ord for char {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if *self < *other {
            Ordering::Less
        } else if *self > *other {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

// Implement PartialOrd for floats (Note: f32/f64 only implement PartialOrd, not Ord)
macro_rules! impl_partial_ord_float {
    ($($t:ty),*) => {
        $(
            impl PartialOrd for $t {
                #[inline]
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    if self.is_nan() || other.is_nan() {
                        None
                    } else if *self < *other {
                        Some(Ordering::Less)
                    } else if *self > *other {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Equal)
                    }
                }

                #[inline]
                fn lt(&self, other: &Self) -> bool {
                    *self < *other
                }

                #[inline]
                fn le(&self, other: &Self) -> bool {
                    *self <= *other
                }

                #[inline]
                fn gt(&self, other: &Self) -> bool {
                    *self > *other
                }

                #[inline]
                fn ge(&self, other: &Self) -> bool {
                    *self >= *other
                }
            }
        )*
    };
}

impl_partial_ord_float!(f32, f64);

// Implement Clone for tuples
impl<A: Clone> Clone for (A,) {
    fn clone(&self) -> Self {
        (self.0.clone(),)
    }
}

impl<A: Clone, B: Clone> Clone for (A, B) {
    fn clone(&self) -> Self {
        (self.0.clone(), self.1.clone())
    }
}

// Implement PartialEq for tuples
impl<A: PartialEq, B: PartialEq> PartialEq for (A, B) {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

// Implement PartialEq for &str
impl PartialEq for str {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

// Implement PartialEq for &str
impl PartialEq for &'_ str {
    fn eq(&self, other: &Self) -> bool {
        *self == *other
    }
}

// Implement Clone for &str
impl Clone for &'_ str {
    fn clone(&self) -> Self {
        *self
    }
}

// ============================================================================
// Hash Trait Implementations
// ============================================================================

impl Hash for i8 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for i16 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for i32 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for i64 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for isize {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for u8 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for u16 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for u32 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for u64 {
    fn hash(&self) -> u64 {
        *self
    }
}

impl Hash for usize {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for bool {
    fn hash(&self) -> u64 {
        if *self { 1 } else { 0 }
    }
}

impl Hash for f32 {
    fn hash(&self) -> u64 {
        // Convert to bits and hash
        self.to_bits().hash()
    }
}

impl Hash for f64 {
    fn hash(&self) -> u64 {
        // Convert to bits and hash
        self.to_bits().hash()
    }
}

impl Hash for char {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for &str {
    fn hash(&self) -> u64 {
        // FNV-1a hash algorithm
        const FNV_PRIME: u64 = 1099511628211;
        const FNV_OFFSET: u64 = 14695981039346656037;

        let mut hash = FNV_OFFSET;
        for byte in self.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}

impl Hash for String {
    fn hash(&self) -> u64 {
        self.as_str().hash()
    }
}

impl<T: Hash> Hash for &T {
    fn hash(&self) -> u64 {
        (*self).hash()
    }
}


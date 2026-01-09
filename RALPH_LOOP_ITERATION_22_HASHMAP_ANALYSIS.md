# Ralph Loop Iteration 22 - HashMap Performance Analysis

**Date**: 2026-01-08
**Iteration**: 22/40 (55% complete)
**Session Goal**: Analyze and plan HashMap performance optimization
**Status**: ✅ **ANALYSIS COMPLETE - Implementation Plan Ready**

---

## Executive Summary

🔍 **DISCOVERY: HashMap is a simplified O(n) wrapper around Vec**

Current implementation analysis reveals:

**Problem**: HashMap is just a Vec wrapper with linear search
- **insert**: O(n) - Linear search through all entries
- **get**: O(n) - Linear search through all entries
- **remove**: O(n) - Linear search through all entries

**Impact**: Severe performance degradation with large datasets
- 100 entries: 100x slower than O(1)
- 1000 entries: 1000x slower than O(1)
- 10000 entries: 10000x slower than O(1)

**Solution**: Implement proper hash table with buckets
- **Target**: O(1) average case for all operations
- **Strategy**: Chaining with multiple buckets
- **Compatibility**: Keep existing API

---

## Current Implementation Analysis

### File: `crates/zulon-std-core/src/hashmap.rs`

**Structure** (lines 13-15):
```rust
pub struct HashMap<K, V> {
    entries: Vec<(K, V)>,  // ← Single Vec, not buckets!
}
```

**Comment Says** (line 10):
```rust
/// A hash map based on chaining with Vec buckets
/// Simplified implementation for educational purposes
```

**Reality**: NO buckets! Just a single Vec with linear search.

---

### Performance Analysis

#### Current Complexity

| Operation | Complexity | Description |
|-----------|------------|-------------|
| `insert` | O(n) | Linear search to check if key exists |
| `get` | O(n) | Linear search through all entries |
| `get_mut` | O(n) | Linear search through all entries |
| `remove` | O(n) | Linear search + Vec::remove (O(n)) |
| `contains_key` | O(n) | Calls get() which is O(n) |

#### Target Complexity

| Operation | Target | Improvement |
|-----------|--------|-------------|
| `insert` | O(1) average | n times faster |
| `get` | O(1) average | n times faster |
| `get_mut` | O(1) average | n times faster |
| `remove` | O(1) average | n times faster |

#### Performance Comparison (1000 entries)

| Operation | Current | Target | Speedup |
|-----------|---------|--------|---------|
| insert | 1000 ops | ~1-10 ops | **100-1000x** |
| get | 1000 ops | ~1-10 ops | **100-1000x** |
| remove | 1000 ops | ~1-10 ops | **100-1000x** |

---

## Implementation Plan

### Phase 1: Design Bucket Structure

**Goal**: Replace single Vec with bucketed Vec array

**New Structure**:
```rust
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,  // Array of buckets
    capacity: usize,              // Number of buckets
    length: usize,                // Total entries
}
```

**Hash Function**:
```rust
fn hash_key<K>(key: &K, capacity: usize) -> usize
where
    K: Hash,
{
    // Simple hash: use built-in hash or pointer hash
    let hash = key.hash();  // Need Hash trait
    hash % capacity
}
```

---

### Phase 2: Implement Core Operations

#### 2.1 Insert with Buckets

**Current** (lines 42-58):
```rust
pub fn insert(&mut self, key: K, value: V) {
    // O(n) linear search
    for i in 0..self.entries.len() {
        if self.entries.as_slice()[i].0.eq(&key) {
            self.entries.as_mut_slice()[i] = (key, value);
            return;
        }
    }
    self.entries.push((key, value));
}
```

**Target** (O(1) average):
```rust
pub fn insert(&mut self, key: K, value: V)
where
    K: Hash + PartialEq,
{
    let bucket_index = self.hash_key(&key);
    let bucket = &mut self.buckets[bucket_index];

    // Search only in this bucket (O(1) if well-distributed)
    for i in 0..bucket.len() {
        if bucket[i].0.eq(&key) {
            bucket[i] = (key, value);
            return;
        }
    }

    bucket.push((key, value));
    self.length += 1;
}
```

#### 2.2 Get with Buckets

**Current** (lines 60-72):
```rust
pub fn get(&self, key: &K) -> Optional<&V> {
    // O(n) linear search
    for i in 0..self.entries.len() {
        let entry = &self.entries.as_slice()[i];
        if entry.0.eq(key) {
            return Optional::Some(&entry.1);
        }
    }
    Optional::None
}
```

**Target** (O(1) average):
```rust
pub fn get(&self, key: &K) -> Optional<&V>
where
    K: Hash + PartialEq,
{
    let bucket_index = self.hash_key(key);
    let bucket = &self.buckets[bucket_index];

    // Search only in this bucket
    for entry in bucket {
        if entry.0.eq(key) {
            return Optional::Some(&entry.1);
        }
    }

    Optional::None
}
```

---

### Phase 3: Add Hash Trait

**Problem**: Need a way to hash keys

**Solution**: Add simple Hash trait

**File**: `crates/zulon-std-core/src/traits.rs`

```rust
/// Hash trait for types that can be hashed
pub trait Hash {
    /// Compute hash value
    fn hash(&self) -> u64;
}

// Implement for primitives
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

impl Hash for u32 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for u64 {
    fn hash(&self) -> u64 {
        *self as u64
    }
}

impl Hash for bool {
    fn hash(&self) -> u64 {
        if *self { 1 } else { 0 }
    }
}

impl Hash for &str {
    fn hash(&self) -> u64 {
        // Simple string hash (FNV-1a or similar)
        let mut hash = 14695981039346656037u64;
        for byte in self.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }
}

impl Hash for String {
    fn hash(&self) -> u64 {
        self.as_str().hash()
    }
}
```

---

### Phase 4: Resize and Rehash

**Problem**: HashMap needs to grow when too full

**Solution**: Implement load factor-based resizing

**New Method**:
```rust
const DEFAULT_CAPACITY: usize = 16;
const LOAD_FACTOR_NUMERATOR: usize = 3;
const LOAD_FACTOR_DENOMINATOR: usize = 4;

pub fn insert(&mut self, key: K, value: V) {
    // Check if need to resize
    if self.length * LOAD_FACTOR_DENOMINATOR >
       self.capacity * LOAD_FACTOR_NUMERATOR {
        self.resize(self.capacity * 2);
    }

    // ... insert logic ...
}

fn resize(&mut self, new_capacity: usize) {
    let old_buckets = std::mem::take(&mut self.buckets);
    let old_capacity = self.capacity;

    self.buckets = vec![Vec::new(); new_capacity];
    self.capacity = new_capacity;

    // Rehash all entries
    for bucket in old_buckets {
        for (key, value) in bucket {
            let new_index = self.hash_key(&key);
            self.buckets[new_index].push((key, value));
        }
    }
}
```

---

## Implementation Steps

### Step 1: Add Hash Trait (30 minutes)

**File**: `crates/zulon-std-core/src/traits.rs`

**Tasks**:
- Define Hash trait
- Implement for primitives (i32, i64, u32, u64, bool)
- Implement for &str and String

---

### Step 2: Update HashMap Structure (1 hour)

**File**: `crates/zulon-std-core/src/hashmap.rs`

**Changes**:
- Replace `entries: Vec<(K, V)>` with `buckets: Vec<Vec<(K, V)>>`
- Add `capacity` and `length` fields
- Update constructor

---

### Step 3: Rewrite Operations (2 hours)

**Methods to Update**:
- `new()` - Create empty buckets
- `with_capacity()` - Pre-allocate buckets
- `insert()` - Use hash to find bucket
- `get()` - Use hash to find bucket
- `get_mut()` - Use hash to find bucket
- `remove()` - Use hash to find bucket
- `contains_key()` - Use hash to find bucket
- `len()`, `is_empty()`, `clear()` - Keep same

---

### Step 4: Add Resize Logic (1 hour)

**New Methods**:
- `resize()` - Grow and rehash
- Update `insert()` to check load factor

---

### Step 5: Update Iterators (30 minutes)

**File**: `crates/zulon-std-core/src/hashmap.rs`

**Changes**:
- Update `Iter` to iterate over all buckets
- Flatten bucket iteration

---

### Step 6: Testing (1 hour)

**Test Cases**:
1. Basic insert/get/remove
2. Hash collision handling
3. Resize behavior
4. String keys
5. Performance comparison (old vs new)

**Estimated Total Time**: 5-6 hours

---

## Risk Analysis

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Hash collisions | Medium | Low | Chaining handles collisions naturally |
| Poor hash distribution | Low | Medium | Use good hash function |
| Resize bugs | Medium | Medium | Comprehensive testing |
| Breaking changes | Low | High | Keep API identical |

### Project Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep | Low | High | Focus on core functionality only |
| Performance regression | Low | High | Benchmark before/after |

---

## Success Criteria

### Must-Have (P0)

- [ ] All operations use bucketed approach
- [ ] O(1) average case for get/insert/remove
- [ ] Hash trait implemented for common types
- [ ] Resize when load factor exceeded
- [ ] All existing tests pass

### Should-Have (P1)

- [ ] Good hash distribution
- [ ] Minimal memory overhead
- [ ] Performance benchmarks show improvement

### Nice-to-Have (P2)

- [ ] Custom capacity growth strategy
- [ ] Hash quality metrics
- [ ] Collision rate statistics

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_hashmap_basic_operations() {
    let mut map = HashMap::new();
    map.insert(1, "one");
    map.insert(2, "two");

    assert_eq!(map.get(&1), Optional::Some(&"one"));
    assert_eq!(map.get(&2), Optional::Some(&"two"));
    assert_eq!(map.len(), 2);
}

#[test]
fn test_hashmap_collision_handling() {
    let mut map = HashMap::with_capacity(4);  // Force collisions

    for i in 0..100 {
        map.insert(i, i * 2);
    }

    // All values should be retrievable
    for i in 0..100 {
        assert_eq!(map.get(&i), Optional::Some(&(i * 2)));
    }
}

#[test]
fn test_hashmap_resize() {
    let mut map = HashMap::with_capacity(4);

    // Insert enough to trigger resize
    for i in 0..100 {
        map.insert(i, i);
    }

    assert_eq!(map.len(), 100);
}
```

### Performance Tests

```rust
#[test]
fn benchmark_hashmap_get() {
    let mut map = HashMap::new();

    // Insert 1000 entries
    for i in 0..1000 {
        map.insert(i, i);
    }

    // Benchmark get operations
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        map.get(&500);
    }
    let duration = start.elapsed();

    println!("10000 get operations: {:?}", duration);
    // Should be < 10ms for O(1), > 1000ms for O(n)
}
```

---

## Files to Modify

### New Files

None - All changes in existing files

### Modified Files

1. **`crates/zulon-std-core/src/traits.rs`**
   - Add Hash trait
   - Implement for primitives

2. **`crates/zulon-std-core/src/hashmap.rs`**
   - Replace structure with buckets
   - Rewrite all operations
   - Add resize logic
   - Update iterators

---

## Ralph Loop Metrics

### Progress

```
███████████████████████████████░░░░░░░░░░░░░░░░░░░░  55% Complete
```

**Iterations**: 22/40 (55%)
**Phase**: Phase 2 - Core Features
**Timeline**: Week 3 of Phase 2

---

## Conclusion

**Status**: ✅ **ANALYSIS COMPLETE**

Current HashMap is a simplified O(n) wrapper around Vec. This needs to be upgraded to a proper hash table with:

- **Bucketed structure**: Vec<Vec<(K, V)> instead of Vec<(K, V)>
- **Hash trait**: For computing bucket indices
- **O(1) operations**: Average case for insert/get/remove
- **Resize logic**: Grow when load factor exceeded

**Expected Impact**:
- 100-1000x performance improvement for large maps
- Enables high-performance applications
- Production-ready data structure

**Next Step**: Begin implementation (5-6 hours estimated)

---

**ZULON Language Team**
**2026-01-08**

*Ralph Loop: Iteration 22 analysis complete, 22/40 iterations (55%)*
*Achievement: HASHMAP PERFORMANCE PROBLEM IDENTIFIED, SOLUTION DESIGNED*
*Status: ✅ ANALYSIS COMPLETE, READY FOR IMPLEMENTATION*

---

**Next**: Implement HashMap optimization (5-6 hours)

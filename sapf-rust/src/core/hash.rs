//! Hash functions for SAPF
//!
//! This module provides various hash functions used throughout the interpreter,
//! including string hashing and integer hashing.

use std::hash::Hasher;

/// Hash a string using the one-at-a-time hash algorithm
/// 
/// This is a very good hash function by Bob Jenkins.
/// Reference: http://www.burtleburtle.net/bob/hash/doobs.html
pub fn hash_str(s: &str) -> i32 {
    let mut hash: i32 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_add(byte as i32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

/// Hash a byte slice using the one-at-a-time hash algorithm
pub fn hash_bytes(bytes: &[u8]) -> i32 {
    let mut hash: i32 = 0;
    for &byte in bytes {
        hash = hash.wrapping_add(byte as i32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

/// Hash a 32-bit integer using Thomas Wang's integer hash
/// 
/// Reference: http://www.concentric.net/~Ttwang/tech/inthash.htm
/// This is a faster hash for integers that provides good distribution.
pub fn hash_i32(key: i32) -> i32 {
    let mut hash = key as u32;
    hash = hash.wrapping_add(!(hash << 15));
    hash ^= hash >> 10;
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 6;
    hash = hash.wrapping_add(!(hash << 11));
    hash ^= hash >> 16;
    hash as i32
}

/// Hash a 64-bit integer using Thomas Wang's 64-bit integer hash
pub fn hash_i64(key: i64) -> i64 {
    let mut hash = key as u64;
    hash ^= (!hash) >> 31;
    hash = hash.wrapping_add(hash << 28);
    hash ^= hash >> 21;
    hash = hash.wrapping_add(hash << 3);
    hash ^= (!hash) >> 5;
    hash = hash.wrapping_add(hash << 13);
    hash ^= hash >> 27;
    hash = hash.wrapping_add(hash << 32);
    hash as i64
}

/// Hash a slice of 32-bit integers
/// 
/// Uses Thomas Wang's integer hash for the combining step.
pub fn hash_i32_slice(slice: &[i32]) -> i32 {
    let mut hash: i32 = 0;
    for &val in slice {
        hash = hash_i32(hash.wrapping_add(val));
    }
    hash
}

/// A hasher implementing the one-at-a-time hash algorithm
/// 
/// This can be used with Rust's standard Hash trait.
#[derive(Default)]
pub struct SapfHasher {
    hash: i32,
}

impl SapfHasher {
    pub fn new() -> Self {
        Self { hash: 0 }
    }
}

impl Hasher for SapfHasher {
    fn finish(&self) -> u64 {
        // Final mixing
        let mut hash = self.hash;
        hash = hash.wrapping_add(hash << 3);
        hash ^= hash >> 11;
        hash = hash.wrapping_add(hash << 15);
        hash as u64
    }
    
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.hash = self.hash.wrapping_add(byte as i32);
            self.hash = self.hash.wrapping_add(self.hash << 10);
            self.hash ^= self.hash >> 6;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_str() {
        // Test empty string
        assert_eq!(hash_str(""), 0);
        
        // Test basic strings
        let h1 = hash_str("hello");
        let h2 = hash_str("world");
        assert_ne!(h1, h2);
        
        // Test that same string produces same hash
        assert_eq!(hash_str("test"), hash_str("test"));
    }
    
    #[test]
    fn test_hash_bytes() {
        assert_eq!(hash_bytes(&[]), 0);
        assert_eq!(hash_bytes(b"hello"), hash_str("hello"));
    }
    
    #[test]
    fn test_hash_i32() {
        // Test that different integers produce different hashes
        assert_ne!(hash_i32(0), hash_i32(1));
        assert_ne!(hash_i32(-1), hash_i32(1));
        
        // Test boundary values
        let _ = hash_i32(i32::MIN);
        let _ = hash_i32(i32::MAX);
    }
    
    #[test]
    fn test_hash_i64() {
        // Test that different integers produce different hashes
        assert_ne!(hash_i64(0), hash_i64(1));
        assert_ne!(hash_i64(-1), hash_i64(1));
        
        // Test boundary values
        let _ = hash_i64(i64::MIN);
        let _ = hash_i64(i64::MAX);
    }
    
    #[test]
    fn test_hash_i32_slice() {
        assert_eq!(hash_i32_slice(&[]), 0);
        
        let h1 = hash_i32_slice(&[1, 2, 3]);
        let h2 = hash_i32_slice(&[3, 2, 1]);
        assert_ne!(h1, h2); // Order matters
        
        // Same slice produces same hash
        assert_eq!(hash_i32_slice(&[1, 2, 3]), hash_i32_slice(&[1, 2, 3]));
    }
    
    #[test]
    fn test_sapf_hasher() {
        use std::hash::Hash;
        
        let mut hasher1 = SapfHasher::new();
        "hello".hash(&mut hasher1);
        let h1 = hasher1.finish();
        
        let mut hasher2 = SapfHasher::new();
        "hello".hash(&mut hasher2);
        let h2 = hasher2.finish();
        
        assert_eq!(h1, h2);
        
        let mut hasher3 = SapfHasher::new();
        "world".hash(&mut hasher3);
        let h3 = hasher3.finish();
        
        assert_ne!(h1, h3);
    }
}
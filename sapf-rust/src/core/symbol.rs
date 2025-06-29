//! Symbol table for SAPF string interning
//!
//! This module provides a thread-safe symbol table for string interning.
//! It ensures that identical strings share the same StringObject instance.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::core::value::{StringObject, Object};

/// Size of the symbol table hash buckets
const SYMBOL_TABLE_SIZE: usize = 4096;
const SYMBOL_TABLE_MASK: usize = SYMBOL_TABLE_SIZE - 1;

/// Thread-safe symbol table for string interning
/// 
/// This provides similar functionality to the C++ getsym function.
/// It ensures that identical strings return the same StringObject instance.
struct SymbolTable {
    // Hash table of symbol buckets
    buckets: Vec<Mutex<HashMap<i32, Arc<StringObject>>>>,
}

impl SymbolTable {
    fn new() -> Self {
        let mut buckets = Vec::with_capacity(SYMBOL_TABLE_SIZE);
        for _ in 0..SYMBOL_TABLE_SIZE {
            buckets.push(Mutex::new(HashMap::new()));
        }
        
        SymbolTable { buckets }
    }
    
    /// Get or create a symbol for the given string
    fn get_symbol(&self, s: &str) -> Arc<StringObject> {
        let string_obj = StringObject::from_str(s);
        let hash = string_obj.hash();
        let bucket_index = (hash as usize) & SYMBOL_TABLE_MASK;
        
        let bucket = &self.buckets[bucket_index];
        let mut map = bucket.lock().unwrap();
        
        // Check if symbol already exists
        if let Some(existing) = map.get(&hash) {
            // Additional check: ensure the strings actually match (hash collision safety)
            if existing.as_str() == s {
                return Arc::clone(existing);
            }
        }
        
        // Create new symbol
        let symbol = Arc::new(string_obj);
        map.insert(hash, Arc::clone(&symbol));
        symbol
    }
    
    /// Look up a symbol without creating it
    fn lookup_symbol(&self, s: &str) -> Option<Arc<StringObject>> {
        let hash = crate::core::hash::hash_str(s);
        let bucket_index = (hash as usize) & SYMBOL_TABLE_MASK;
        
        let bucket = &self.buckets[bucket_index];
        let map = bucket.lock().unwrap();
        
        if let Some(existing) = map.get(&hash) {
            if existing.as_str() == s {
                return Some(Arc::clone(existing));
            }
        }
        
        None
    }
    
    /// Get statistics about the symbol table
    fn stats(&self) -> SymbolTableStats {
        let mut total_symbols = 0;
        let mut used_buckets = 0;
        let mut max_bucket_size = 0;
        
        for bucket in &self.buckets {
            let map = bucket.lock().unwrap();
            let bucket_size = map.len();
            total_symbols += bucket_size;
            
            if bucket_size > 0 {
                used_buckets += 1;
                max_bucket_size = max_bucket_size.max(bucket_size);
            }
        }
        
        SymbolTableStats {
            total_symbols,
            used_buckets,
            total_buckets: SYMBOL_TABLE_SIZE,
            max_bucket_size,
            load_factor: total_symbols as f64 / SYMBOL_TABLE_SIZE as f64,
        }
    }
}

/// Statistics about symbol table usage
#[derive(Debug)]
pub struct SymbolTableStats {
    pub total_symbols: usize,
    pub used_buckets: usize,
    pub total_buckets: usize,
    pub max_bucket_size: usize,
    pub load_factor: f64,
}

use std::sync::OnceLock;

// Global symbol table instance using OnceLock for thread safety
static SYMBOL_TABLE: OnceLock<SymbolTable> = OnceLock::new();

/// Get reference to the global symbol table
fn get_symbol_table() -> &'static SymbolTable {
    SYMBOL_TABLE.get_or_init(|| SymbolTable::new())
}

/// Get or create a symbol for the given string
/// 
/// This is the main entry point for string interning, corresponding to
/// the getsym function in the C++ implementation.
pub fn get_symbol(s: &str) -> Arc<StringObject> {
    get_symbol_table().get_symbol(s)
}

/// Look up a symbol without creating it
/// 
/// Returns None if the symbol doesn't exist in the table.
pub fn lookup_symbol(s: &str) -> Option<Arc<StringObject>> {
    get_symbol_table().lookup_symbol(s)
}

/// Get statistics about the global symbol table
pub fn symbol_table_stats() -> SymbolTableStats {
    get_symbol_table().stats()
}

/// Clear the symbol table (for testing)
/// 
/// Note: This doesn't actually clear the global table since OnceLock doesn't support it.
/// Instead, we'll create a fresh symbol table for each test by creating local instances.
#[cfg(test)]
pub fn clear_symbol_table() {
    // OnceLock doesn't support clearing, so tests will share the global table
    // For isolated testing, we would need a different approach
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_interning() {
        let sym1 = get_symbol("test_hello_unique");
        let sym2 = get_symbol("test_hello_unique");
        let sym3 = get_symbol("test_world_unique");
        
        // Same string should return the same object
        assert!(Arc::ptr_eq(&sym1, &sym2));
        
        // Different strings should return different objects
        assert!(!Arc::ptr_eq(&sym1, &sym3));
        
        // Content should be correct
        assert_eq!(sym1.as_str(), "test_hello_unique");
        assert_eq!(sym3.as_str(), "test_world_unique");
    }
    
    #[test]
    fn test_symbol_lookup() {
        let unique_str = "test_lookup_unique_12345";
        
        // Lookup non-existent symbol first
        assert!(lookup_symbol("definitely_nonexistent_98765").is_none());
        
        // Create a symbol
        let sym1 = get_symbol(unique_str);
        
        // Now lookup should find it
        let sym2 = lookup_symbol(unique_str).unwrap();
        assert!(Arc::ptr_eq(&sym1, &sym2));
        
        // Lookup still returns None for non-existent
        assert!(lookup_symbol("still_nonexistent_67890").is_none());
    }
    
    #[test]
    fn test_hash_collision_handling() {
        // Test with unique strings that might have the same hash
        // (though this is unlikely with a good hash function)
        let strings = ["hash_test_a", "hash_test_b", "hash_test_c", 
                      "hash_test_collision", "hash_test_unique"];
        let mut symbols = Vec::new();
        
        for s in &strings {
            symbols.push(get_symbol(s));
        }
        
        // All symbols should be interned correctly
        for (i, s) in strings.iter().enumerate() {
            assert_eq!(symbols[i].as_str(), *s);
            
            // Getting the same string again should return the same object
            let same_symbol = get_symbol(s);
            assert!(Arc::ptr_eq(&symbols[i], &same_symbol));
        }
    }
    
    #[test]
    fn test_symbol_table_stats() {
        let initial_stats = symbol_table_stats();
        let initial_count = initial_stats.total_symbols;
        
        // Add some unique symbols
        get_symbol("stats_test_one");
        get_symbol("stats_test_two");
        get_symbol("stats_test_three");
        
        let stats = symbol_table_stats();
        assert!(stats.total_symbols >= initial_count + 3);
        assert!(stats.used_buckets > 0);
        assert!(stats.load_factor > 0.0);
        
        // Adding the same symbol shouldn't increase count
        get_symbol("stats_test_one");
        let stats2 = symbol_table_stats();
        assert_eq!(stats2.total_symbols, stats.total_symbols);
    }
    
    #[test]
    fn test_empty_string_symbol() {
        let empty1 = get_symbol("");
        let empty2 = get_symbol("");
        
        assert!(Arc::ptr_eq(&empty1, &empty2));
        assert_eq!(empty1.as_str(), "");
        assert!(empty1.is_zero()); // Empty strings are considered zero
    }
    
    #[test]
    fn test_unicode_symbols() {
        let unicode_strings = ["🎵_test", "α_test", "中文_test", "🔊🎶_test"];
        
        for s in &unicode_strings {
            let sym1 = get_symbol(s);
            let sym2 = get_symbol(s);
            
            assert!(Arc::ptr_eq(&sym1, &sym2));
            assert_eq!(sym1.as_str(), *s);
        }
    }
}
//! Set operations for SAPF
//!
//! This module implements set theory operations (union, intersection, difference, etc.)
//! and collection utilities. It provides a Set type for efficient membership testing
//! and set operations on Lists.

use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fmt;
use crate::core::error::{SapfError, Result};
use crate::core::value::{Value, Object};
use crate::core::list::{List, Array};
use crate::vm::Thread;

/// Set pair to track both value and original index
#[derive(Debug, Clone)]
struct SetPair {
    value: Value,
    index: i32,
}

/// Hash-based Set implementation for efficient membership testing
/// 
/// This corresponds to the Set class in the C++ implementation.
/// Uses open addressing with linear probing for collision resolution.
#[derive(Debug, Clone)]
pub struct Set {
    size: usize,
    capacity: usize,
    indices: Vec<i32>,  // Hash table indices (0 = empty, >0 = index+1 into pairs)
    pairs: Vec<SetPair>, // Actual data storage
}

impl Set {
    /// Create a new set with the specified initial capacity
    pub fn new(capacity: usize) -> Self {
        let cap = next_power_of_two(capacity.max(32));
        Self {
            size: 0,
            capacity: cap,
            indices: vec![0; cap * 2], // Hash table is 2x capacity for good performance
            pairs: Vec::with_capacity(cap),
        }
    }

    /// Create a set from a list
    pub fn from_list(thread: &mut Thread, list: Arc<List>) -> Result<Self> {
        // For now, we assume all lists are finite (would need lazy evaluation support for infinite lists)
        let mut set = Self::new(32);
        set.put_all(thread, list)?;
        Ok(set)
    }

    /// Get the number of unique elements in the set
    pub fn size(&self) -> usize {
        self.size
    }

    /// Check if the set contains a value
    pub fn has(&self, thread: &mut Thread, value: &Value) -> Result<bool> {
        let hash = self.hash_value(value);
        let mask = (self.capacity * 2 - 1) as usize;
        let mut index = (hash as usize) & mask;
        
        loop {
            let index2 = self.indices[index] - 1;
            if index2 == -1 {
                return Ok(false);
            }
            
            let test_val = &self.pairs[index2 as usize].value;
            if value.equals(test_val)? {
                return Ok(true);
            }
            
            index = (index + 1) & mask;
        }
    }

    /// Find the original index of a value in the set (-1 if not found)
    pub fn index_of(&self, thread: &mut Thread, value: &Value) -> Result<i32> {
        let hash = self.hash_value(value);
        let mask = (self.capacity * 2 - 1) as usize;
        let mut index = (hash as usize) & mask;
        
        loop {
            let index2 = self.indices[index] - 1;
            if index2 == -1 {
                return Ok(-1);
            }
            
            let test_val = &self.pairs[index2 as usize].value;
            if value.equals(test_val)? {
                return Ok(self.pairs[index2 as usize].index);
            }
            
            index = (index + 1) & mask;
        }
    }

    /// Get value at index i (for iteration)
    pub fn at(&self, i: usize) -> Option<&Value> {
        if i < self.size {
            Some(&self.pairs[i].value)
        } else {
            None
        }
    }

    /// Add a value with its original index to the set
    fn put(&mut self, thread: &mut Thread, value: Value, original_index: i32) -> Result<()> {
        if self.size == self.capacity {
            self.grow(thread)?;
        }

        let hash = self.hash_value(&value);
        let mask = (self.capacity * 2 - 1) as usize;
        let mut index = (hash as usize) & mask;
        
        loop {
            let index2 = self.indices[index] - 1;
            if index2 == -1 {
                // Empty slot, insert here
                let pair_index = self.size;
                self.indices[index] = (pair_index + 1) as i32;
                self.pairs.push(SetPair { value, index: original_index });
                self.size += 1;
                return Ok(());
            }
            
            let test_val = &self.pairs[index2 as usize].value;
            if value.equals(test_val)? {
                // Value already exists, don't add duplicate
                return Ok(());
            }
            
            index = (index + 1) & mask;
        }
    }

    /// Add all values from a list to the set
    fn put_all(&mut self, thread: &mut Thread, list: Arc<List>) -> Result<()> {
        // We need to make a mutable copy to call length() since it requires &mut self
        let mut list_copy = (*list).clone();
        let length = list_copy.length()?;
        
        for i in 0..length {
            if let Ok(val) = list.at(i) {
                self.put(thread, val, i as i32)?;
            }
        }
        Ok(())
    }

    /// Convert set to a Value list
    pub fn as_v_list(&self) -> Arc<List> {
        let mut values = Vec::with_capacity(self.size);
        for i in 0..self.size {
            values.push(self.pairs[i].value.clone());
        }
        let array = Array::from_values(values);
        Arc::new(List::from_array(array))
    }

    /// Convert set to a Real list (if all values are numeric)
    pub fn as_z_list(&self) -> Result<Arc<List>> {
        let mut reals = Vec::with_capacity(self.size);
        for i in 0..self.size {
            if let Value::Real(r) = &self.pairs[i].value {
                reals.push(*r);
            } else {
                return Err(SapfError::InvalidValue("Cannot convert non-numeric set to ZList".to_string()));
            }
        }
        let array = Array::from_reals(reals);
        Ok(Arc::new(List::from_array(array)))
    }

    /// Hash a value for the hash table
    fn hash_value(&self, value: &Value) -> u32 {
        let mut hasher = DefaultHasher::new();
        // Use the value's built-in hash method if available
        match value {
            Value::Real(r) => r.to_bits().hash(&mut hasher),
            Value::Object(obj) => {
                // Hash based on string representation for objects
                format!("{}", obj).hash(&mut hasher)
            },
            Value::Nil => 0_u32.hash(&mut hasher),
        }
        hasher.finish() as u32
    }

    /// Grow the hash table when it gets full
    fn grow(&mut self, thread: &mut Thread) -> Result<()> {
        let old_pairs = std::mem::take(&mut self.pairs);
        let old_size = self.size;

        // Double capacity
        self.capacity *= 2;
        self.size = 0;
        self.indices = vec![0; self.capacity * 2];
        self.pairs = Vec::with_capacity(self.capacity);

        // Re-insert all elements
        for pair in old_pairs.into_iter().take(old_size) {
            self.put(thread, pair.value, pair.index)?;
        }

        Ok(())
    }
}

impl Object for Set {
    fn as_float(&self) -> Result<f64> {
        Err(SapfError::WrongType)
    }

    fn deref(&self) -> Result<Value> {
        Ok(Value::Object(Arc::new(self.clone())))
    }

    fn type_name(&self) -> &'static str {
        "Set"
    }

    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Set({})", self.size)
    }
}

/// Find next power of two for capacity calculation
fn next_power_of_two(n: usize) -> usize {
    if n == 0 { return 1; }
    let mut power = 1;
    while power < n {
        power <<= 1;
    }
    power
}

// ============================================================================
// Set Operation Functions
// ============================================================================

/// Remove duplicates from a list (nub operation)
pub fn nub(thread: &mut Thread, list: Arc<List>) -> Result<Arc<List>> {
    let set = Set::from_list(thread, list)?;
    Ok(set.as_v_list())
}

/// Set union (OR operation)
pub fn set_or(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<Arc<List>> {
    let mut set = Set::new(32);
    set.put_all(thread, a.clone())?;
    set.put_all(thread, b.clone())?;
    
    // Return ZList if both inputs are ZList, otherwise VList
    if a.is_real_list() && b.is_real_list() {
        Ok(set.as_z_list()?)
    } else {
        Ok(set.as_v_list())
    }
}

/// Set intersection (AND operation)
pub fn set_and(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<Arc<List>> {
    let set_a = Set::from_list(thread, a.clone())?;
    let set_b = Set::from_list(thread, b.clone())?;
    
    let mut result_values = Vec::new();
    
    for i in 0..set_a.size() {
        if let Some(v) = set_a.at(i) {
            if set_b.has(thread, v)? {
                result_values.push(v.clone());
            }
        }
    }
    
    if a.is_real_list() && b.is_real_list() {
        let mut reals = Vec::new();
        for val in result_values {
            if let Value::Real(r) = val {
                reals.push(r);
            } else {
                return Err(SapfError::InvalidValue("Non-numeric value in ZList intersection".to_string()));
            }
        }
        let array = Array::from_reals(reals);
        Ok(Arc::new(List::from_array(array)))
    } else {
        let array = Array::from_values(result_values);
        Ok(Arc::new(List::from_array(array)))
    }
}

/// Set difference (MINUS operation)
pub fn set_minus(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<Arc<List>> {
    let set_a = Set::from_list(thread, a.clone())?;
    let set_b = Set::from_list(thread, b.clone())?;
    
    let mut result_values = Vec::new();
    
    for i in 0..set_a.size() {
        if let Some(v) = set_a.at(i) {
            if !set_b.has(thread, v)? {
                result_values.push(v.clone());
            }
        }
    }
    
    if a.is_real_list() && b.is_real_list() {
        let mut reals = Vec::new();
        for val in result_values {
            if let Value::Real(r) = val {
                reals.push(r);
            } else {
                return Err(SapfError::InvalidValue("Non-numeric value in ZList difference".to_string()));
            }
        }
        let array = Array::from_reals(reals);
        Ok(Arc::new(List::from_array(array)))
    } else {
        let array = Array::from_values(result_values);
        Ok(Arc::new(List::from_array(array)))
    }
}

/// Set symmetric difference (XOR operation)  
pub fn set_xor(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<Arc<List>> {
    let set_a = Set::from_list(thread, a.clone())?;
    let set_b = Set::from_list(thread, b.clone())?;
    
    let mut result_values = Vec::new();
    
    // Add elements from A that are not in B
    for i in 0..set_a.size() {
        if let Some(v) = set_a.at(i) {
            if !set_b.has(thread, v)? {
                result_values.push(v.clone());
            }
        }
    }
    
    // Add elements from B that are not in A
    for i in 0..set_b.size() {
        if let Some(v) = set_b.at(i) {
            if !set_a.has(thread, v)? {
                result_values.push(v.clone());
            }
        }
    }
    
    if a.is_real_list() && b.is_real_list() {
        let mut reals = Vec::new();
        for val in result_values {
            if let Value::Real(r) = val {
                reals.push(r);
            } else {
                return Err(SapfError::InvalidValue("Non-numeric value in ZList XOR".to_string()));
            }
        }
        let array = Array::from_reals(reals);
        Ok(Arc::new(List::from_array(array)))
    } else {
        let array = Array::from_values(result_values);
        Ok(Arc::new(List::from_array(array)))
    }
}

/// Test if set A is a subset of set B
pub fn subset(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<bool> {
    let set_a = Set::from_list(thread, a)?;
    let set_b = Set::from_list(thread, b)?;
    
    for i in 0..set_a.size() {
        if let Some(v) = set_a.at(i) {
            if !set_b.has(thread, v)? {
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

/// Test if two sets are equal
pub fn set_equals(thread: &mut Thread, a: Arc<List>, b: Arc<List>) -> Result<bool> {
    let set_a = Set::from_list(thread, a)?;
    let set_b = Set::from_list(thread, b)?;
    
    // Two sets are equal if they have the same size and contain the same elements
    if set_a.size() != set_b.size() {
        return Ok(false);
    }
    
    for i in 0..set_a.size() {
        if let Some(v) = set_a.at(i) {
            if !set_b.has(thread, v)? {
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

/// Find index of item(s) in a list
pub fn find(thread: &mut Thread, item: Value, list: Arc<List>) -> Result<Value> {
    let set = Set::from_list(thread, list)?;
    
    match &item {
        Value::Object(obj) => {
            if let Some(item_list) = obj.as_any().downcast_ref::<List>() {
                // Handle list of items - return list of indices
                let mut list_copy = item_list.clone();
                let length = list_copy.length()?;
                let mut result_values = Vec::new();
                
                for i in 0..length {
                    if let Ok(val) = item_list.at(i) {
                        let index = set.index_of(thread, &val)?;
                        result_values.push(Value::Real(index as f64));
                    }
                }
                
                if item_list.is_real_list() {
                    let mut reals = Vec::new();
                    for val in result_values {
                        if let Value::Real(r) = val {
                            reals.push(r);
                        }
                    }
                    let array = Array::from_reals(reals);
                    Ok(Value::Object(Arc::new(List::from_array(array))))
                } else {
                    let array = Array::from_values(result_values);
                    Ok(Value::Object(Arc::new(List::from_array(array))))
                }
            } else {
                // Single item
                let index = set.index_of(thread, &item)?;
                Ok(Value::Real(index as f64))
            }
        }
        _ => {
            // Single item
            let index = set.index_of(thread, &item)?;
            Ok(Value::Real(index as f64))
        }
    }
}

/// Test if list contains item(s)
pub fn set_has(thread: &mut Thread, item: Value, list: Arc<List>) -> Result<Value> {
    let set = Set::from_list(thread, list)?;
    
    match &item {
        Value::Object(obj) => {
            if let Some(item_list) = obj.as_any().downcast_ref::<List>() {
                // Handle list of items - return list of booleans
                let mut list_copy = item_list.clone();
                let length = list_copy.length()?;
                let mut result_values = Vec::new();
                
                for i in 0..length {
                    if let Ok(val) = item_list.at(i) {
                        let has_val = set.has(thread, &val)?;
                        result_values.push(Value::Real(if has_val { 1.0 } else { 0.0 }));
                    }
                }
                
                if item_list.is_real_list() {
                    let mut reals = Vec::new();
                    for val in result_values {
                        if let Value::Real(r) = val {
                            reals.push(r);
                        }
                    }
                    let array = Array::from_reals(reals);
                    Ok(Value::Object(Arc::new(List::from_array(array))))
                } else {
                    let array = Array::from_values(result_values);
                    Ok(Value::Object(Arc::new(List::from_array(array))))
                }
            } else {
                // Single item
                let has_val = set.has(thread, &item)?;
                Ok(Value::Real(if has_val { 1.0 } else { 0.0 }))
            }
        }
        _ => {
            // Single item
            let has_val = set.has(thread, &item)?;
            Ok(Value::Real(if has_val { 1.0 } else { 0.0 }))
        }
    }
}

// ============================================================================
// Built-in Function Wrappers
// ============================================================================

/// Helper function to extract a List from a Value
fn extract_list(value: Value) -> Result<Arc<List>> {
    match value {
        Value::Object(obj) => {
            if let Some(list) = obj.as_any().downcast_ref::<List>() {
                Ok(Arc::new(list.clone()))
            } else {
                Err(SapfError::WrongType)
            }
        }
        _ => Err(SapfError::WrongType),
    }
}

/// Built-in function implementations for registration with VM
pub mod builtins {
    use super::*;
    use crate::vm::Thread;

    pub fn nub_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let value = thread.pop()?;
        let list = extract_list(value)?;
        
        // For now, we assume all lists are finite (would need lazy evaluation support for infinite lists)
        
        let result = nub(thread, list)?;
        Ok(vec![Value::Object(result)])
    }

    pub fn set_or_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = set_or(thread, a, b)?;
        Ok(vec![Value::Object(result)])
    }

    pub fn set_and_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = set_and(thread, a, b)?;
        Ok(vec![Value::Object(result)])
    }

    pub fn set_xor_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = set_xor(thread, a, b)?;
        Ok(vec![Value::Object(result)])
    }

    pub fn set_minus_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = set_minus(thread, a, b)?;
        Ok(vec![Value::Object(result)])
    }

    pub fn subset_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = subset(thread, a, b)?;
        Ok(vec![Value::Real(if result { 1.0 } else { 0.0 })])
    }

    pub fn set_equals_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let b_val = thread.pop()?;
        let b = extract_list(b_val)?;
        // For now, we assume all lists are finite
        
        let a_val = thread.pop()?;
        let a = extract_list(a_val)?;
        // For now, we assume all lists are finite
        
        let result = set_equals(thread, a, b)?;
        Ok(vec![Value::Real(if result { 1.0 } else { 0.0 })])
    }

    pub fn find_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let list_val = thread.pop()?;
        let list = extract_list(list_val)?;
        // For now, we assume all lists are finite (would need lazy evaluation support for infinite lists)
        // TODO: Add proper infinite list detection when lazy evaluation is implemented
        if false {
            return Err(SapfError::InvalidValue("find : list must be finite".to_string()));
        }
        
        let item = thread.pop()?;
        let result = find(thread, item, list)?;
        Ok(vec![result])
    }

    pub fn set_has_prim(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>> {
        let list_val = thread.pop()?;
        let list = extract_list(list_val)?;
        // For now, we assume all lists are finite (would need lazy evaluation support for infinite lists)
        // TODO: Add proper infinite list detection when lazy evaluation is implemented
        if false {
            return Err(SapfError::InvalidValue("Shas : list must be finite".to_string()));
        }
        
        let item = thread.pop()?;
        let result = set_has(thread, item, list)?;
        Ok(vec![result])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{VM, Thread};

    #[test]
    fn test_set_creation() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array = Array::from_values(vec![
            Value::Real(1.0),
            Value::Real(2.0),
            Value::Real(1.0), // duplicate
            Value::Real(3.0),
        ]);
        let list = Arc::new(List::from_array(array));
        
        let set = Set::from_list(&mut thread, list).unwrap();
        assert_eq!(set.size(), 3); // Should have removed duplicate
    }

    #[test]
    fn test_nub_operation() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array = Array::from_values(vec![
            Value::Real(1.0),
            Value::Real(2.0),
            Value::Real(1.0), // duplicate
            Value::Real(3.0),
            Value::Real(2.0), // duplicate
        ]);
        let list = Arc::new(List::from_array(array));
        
        let result = nub(&mut thread, list).unwrap();
        let mut result_copy = (*result).clone();
        assert_eq!(result_copy.length().unwrap(), 3);
    }

    #[test]
    fn test_set_union() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array_a = Array::from_values(vec![Value::Real(1.0), Value::Real(2.0)]);
        let list_a = Arc::new(List::from_array(array_a));
        let array_b = Array::from_values(vec![Value::Real(2.0), Value::Real(3.0)]);
        let list_b = Arc::new(List::from_array(array_b));
        
        let result = set_or(&mut thread, list_a, list_b).unwrap();
        let mut result_copy = (*result).clone();
        assert_eq!(result_copy.length().unwrap(), 3); // {1, 2, 3}
    }

    #[test]
    fn test_set_intersection() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array_a = Array::from_values(vec![Value::Real(1.0), Value::Real(2.0), Value::Real(3.0)]);
        let list_a = Arc::new(List::from_array(array_a));
        let array_b = Array::from_values(vec![Value::Real(2.0), Value::Real(3.0), Value::Real(4.0)]);
        let list_b = Arc::new(List::from_array(array_b));
        
        let result = set_and(&mut thread, list_a, list_b).unwrap();
        let mut result_copy = (*result).clone();
        assert_eq!(result_copy.length().unwrap(), 2); // {2, 3}
    }

    #[test]
    fn test_subset() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array_a = Array::from_values(vec![Value::Real(1.0), Value::Real(2.0)]);
        let list_a = Arc::new(List::from_array(array_a));
        let array_b = Array::from_values(vec![Value::Real(1.0), Value::Real(2.0), Value::Real(3.0)]);
        let list_b = Arc::new(List::from_array(array_b));
        
        let result = subset(&mut thread, list_a, list_b).unwrap();
        assert!(result); // {1, 2} is subset of {1, 2, 3}
    }

    #[test]
    fn test_find_operation() {
        let mut vm = VM::new();
        let mut thread = Thread::new();
        
        let array = Array::from_values(vec![
            Value::Real(10.0),
            Value::Real(20.0),
            Value::Real(30.0),
        ]);
        let list = Arc::new(List::from_array(array));
        
        let result = find(&mut thread, Value::Real(20.0), list).unwrap();
        if let Value::Real(index) = result {
            assert_eq!(index, 1.0); // Found at index 1
        } else {
            panic!("Expected real result");
        }
    }
}
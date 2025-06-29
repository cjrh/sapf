//! Form system for SAPF
//!
//! This module implements the Form and GForm types which provide key-value mappings
//! with inheritance support. Forms are the foundation of SAPF's object system,
//! supporting multiple inheritance with Dylan-style method resolution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::rc::Rc;
use crate::core::value::{Value, Object};
use crate::core::error::{SapfError, Result};
use crate::core::hash::hash_str;

/// A key-value mapping that can be used in forms
/// This corresponds to the Table class in the C++ implementation
#[derive(Debug, Clone)]
pub struct Table {
    /// Hash table storing key-value pairs
    map: HashMap<i32, (Value, Value)>, // (key, value) pairs indexed by hash
    /// Cached size for efficiency
    size: usize,
}

impl Table {
    /// Create a new empty table
    pub fn new() -> Self {
        Table {
            map: HashMap::new(),
            size: 0,
        }
    }
    
    /// Create a table from key-value pairs
    pub fn from_pairs(pairs: Vec<(Value, Value)>) -> Self {
        let mut map = HashMap::new();
        let size = pairs.len();
        
        for (key, value) in pairs {
            let hash = Self::compute_hash(&key);
            map.insert(hash, (key, value));
        }
        
        Table { map, size }
    }
    
    /// Get a value by key
    pub fn get(&self, key: &Value) -> Option<&Value> {
        let hash = Self::compute_hash(key);
        self.map.get(&hash).and_then(|(stored_key, value)| {
            // Check for hash collision
            if Self::values_equal(stored_key, key) {
                Some(value)
            } else {
                None
            }
        })
    }
    
    /// Check if the table contains a key
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }
    
    /// Get the number of key-value pairs
    pub fn len(&self) -> usize {
        self.size
    }
    
    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    
    /// Insert a key-value pair, creating a new table (immutable operation)
    pub fn put(&self, key: Value, value: Value) -> Self {
        let mut new_map = self.map.clone();
        let hash = Self::compute_hash(&key);
        
        let new_size = if new_map.contains_key(&hash) {
            self.size // Replacing existing key
        } else {
            self.size + 1 // Adding new key
        };
        
        new_map.insert(hash, (key, value));
        
        Table {
            map: new_map,
            size: new_size,
        }
    }
    
    /// Get all key-value pairs as a vector
    pub fn pairs(&self) -> Vec<(Value, Value)> {
        self.map.values().cloned().collect()
    }
    
    /// Compute hash for a key
    fn compute_hash(key: &Value) -> i32 {
        match key {
            Value::Real(f) => {
                // Convert float to bits for consistent hashing
                let bits = f.to_bits();
                (bits ^ (bits >> 32)) as i32
            }
            Value::Object(obj) => {
                // Use the object's string representation for hashing
                hash_str(&format!("{}", obj))
            }
        }
    }
    
    /// Check if two values are equal (handling potential hash collisions)
    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Real(a), Value::Real(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => {
                // For objects, we compare their string representations
                // This is a simplification - ideally we'd have proper equality
                format!("{}", a) == format!("{}", b)
            }
            _ => false,
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// A thread-safe table for use in GForm
/// This corresponds to the GTable class in the C++ implementation
#[derive(Debug)]
pub struct GTable {
    /// Thread-safe hash table
    map: Arc<RwLock<HashMap<i32, (Value, Value)>>>,
    /// Atomic size counter
    size: Arc<Mutex<usize>>,
}

impl GTable {
    /// Create a new empty thread-safe table
    pub fn new() -> Self {
        GTable {
            map: Arc::new(RwLock::new(HashMap::new())),
            size: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Get a value by key (thread-safe)
    pub fn get(&self, key: &Value) -> Option<Value> {
        let hash = Table::compute_hash(key);
        let map = self.map.read().unwrap();
        map.get(&hash).and_then(|(stored_key, value)| {
            if Table::values_equal(stored_key, key) {
                Some(value.clone())
            } else {
                None
            }
        })
    }
    
    /// Insert a key-value pair (thread-safe, mutable operation)
    pub fn put_impure(&self, key: Value, value: Value) -> Result<()> {
        let hash = Table::compute_hash(&key);
        let mut map = self.map.write().unwrap();
        
        let is_new_key = !map.contains_key(&hash);
        map.insert(hash, (key, value));
        
        if is_new_key {
            let mut size = self.size.lock().unwrap();
            *size += 1;
        }
        
        Ok(())
    }
    
    /// Create a new GTable with an additional key-value pair (immutable operation)
    pub fn put_pure(&self, key: Value, value: Value) -> Self {
        let new_gtable = GTable::new();
        
        // Copy existing entries
        {
            let map = self.map.read().unwrap();
            let mut new_map = new_gtable.map.write().unwrap();
            let mut new_size = new_gtable.size.lock().unwrap();
            
            for (hash, (k, v)) in map.iter() {
                new_map.insert(*hash, (k.clone(), v.clone()));
            }
            *new_size = map.len();
        }
        
        // Add the new entry
        new_gtable.put_impure(key, value).unwrap();
        new_gtable
    }
    
    /// Get the number of key-value pairs
    pub fn len(&self) -> usize {
        *self.size.lock().unwrap()
    }
    
    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Clone for GTable {
    fn clone(&self) -> Self {
        let new_gtable = GTable::new();
        
        // Scope the borrows to avoid conflicts
        {
            let map = self.map.read().unwrap();
            let mut new_map = new_gtable.map.write().unwrap();
            let mut new_size = new_gtable.size.lock().unwrap();
            
            for (hash, (k, v)) in map.iter() {
                new_map.insert(*hash, (k.clone(), v.clone()));
            }
            *new_size = map.len();
        }
        
        new_gtable
    }
}

impl Default for GTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable form with inheritance support
/// This corresponds to the Form class in the C++ implementation
#[derive(Debug, Clone)]
pub struct Form {
    /// The key-value table for this form level
    table: Table,
    /// The parent form in the inheritance chain
    parent: Option<Rc<Form>>,
}

impl Form {
    /// Create a new empty form
    pub fn new() -> Self {
        Form {
            table: Table::new(),
            parent: None,
        }
    }
    
    /// Create a form from a table
    pub fn from_table(table: Table) -> Self {
        Form {
            table,
            parent: None,
        }
    }
    
    /// Create a form with a parent
    pub fn with_parent(table: Table, parent: Rc<Form>) -> Self {
        Form {
            table,
            parent: Some(parent),
        }
    }
    
    /// Get a value by key, searching through the inheritance chain
    pub fn get(&self, key: &Value) -> Option<Value> {
        // First check this form's table
        if let Some(value) = self.table.get(key) {
            return Some(value.clone());
        }
        
        // Then check parent forms
        if let Some(parent) = &self.parent {
            parent.get(key)
        } else {
            None
        }
    }
    
    /// Check if a key exists in this form or its parents
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }
    
    /// Get a value, returning an error if not found
    pub fn must_get(&self, key: &Value) -> Result<Value> {
        self.get(key).ok_or(SapfError::NotFound)
    }
    
    /// Create a new form with an additional key-value pair
    pub fn put(&self, key: Value, value: Value) -> Self {
        let new_table = self.table.put(key, value);
        Form {
            table: new_table,
            parent: self.parent.clone(),
        }
    }
    
    /// Get the immediate table (not including parents)
    pub fn table(&self) -> &Table {
        &self.table
    }
    
    /// Get the parent form
    pub fn parent(&self) -> Option<&Rc<Form>> {
        self.parent.as_ref()
    }
    
    /// Get all key-value pairs including inherited ones
    pub fn all_pairs(&self) -> Vec<(Value, Value)> {
        let mut pairs = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();
        
        self.collect_pairs(&mut pairs, &mut seen_keys);
        pairs
    }
    
    /// Helper to collect pairs with inheritance resolution
    fn collect_pairs(&self, pairs: &mut Vec<(Value, Value)>, seen_keys: &mut std::collections::HashSet<i32>) {
        // Add pairs from this level
        for (key, value) in self.table.pairs() {
            let hash = Table::compute_hash(&key);
            if !seen_keys.contains(&hash) {
                seen_keys.insert(hash);
                pairs.push((key, value));
            }
        }
        
        // Add pairs from parent
        if let Some(parent) = &self.parent {
            parent.collect_pairs(pairs, seen_keys);
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Object for Form {
    fn as_float(&self) -> Result<f64> {
        // Forms cannot be converted to float
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Object(self.clone_object()))
    }
    
    fn type_name(&self) -> &'static str {
        "Form"
    }
    
    fn clone_object(&self) -> Rc<dyn Object> {
        Rc::new(self.clone())
    }
    
    fn is_zero(&self) -> bool {
        self.table.is_empty() && self.parent.is_none()
    }
}

impl std::fmt::Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let pairs = self.all_pairs();
        for (i, (key, value)) in pairs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", key, value)?;
        }
        write!(f, "}}")
    }
}

/// Mutable form with thread-safe inheritance support
/// This corresponds to the GForm class in the C++ implementation
#[derive(Debug)]
pub struct GForm {
    /// The key-value table for this form level
    table: Arc<GTable>,
    /// The parent form in the inheritance chain
    parent: Option<Arc<GForm>>,
}

impl GForm {
    /// Create a new empty mutable form
    pub fn new() -> Self {
        GForm {
            table: Arc::new(GTable::new()),
            parent: None,
        }
    }
    
    /// Create a form from a table
    pub fn from_table(table: GTable) -> Self {
        GForm {
            table: Arc::new(table),
            parent: None,
        }
    }
    
    /// Create a form with a parent
    pub fn with_parent(table: GTable, parent: Arc<GForm>) -> Self {
        GForm {
            table: Arc::new(table),
            parent: Some(parent),
        }
    }
    
    /// Get a value by key, searching through the inheritance chain
    pub fn get(&self, key: &Value) -> Option<Value> {
        // First check this form's table
        if let Some(value) = self.table.get(key) {
            return Some(value);
        }
        
        // Then check parent forms
        if let Some(parent) = &self.parent {
            parent.get(key)
        } else {
            None
        }
    }
    
    /// Check if a key exists in this form or its parents
    pub fn contains_key(&self, key: &Value) -> bool {
        self.get(key).is_some()
    }
    
    /// Get a value, returning an error if not found
    pub fn must_get(&self, key: &Value) -> Result<Value> {
        self.get(key).ok_or(SapfError::NotFound)
    }
    
    /// Insert a key-value pair (mutable operation)
    pub fn put_impure(&self, key: Value, value: Value) -> Result<()> {
        self.table.put_impure(key, value)
    }
    
    /// Create a new form with an additional key-value pair (immutable operation)
    pub fn put_pure(&self, key: Value, value: Value) -> Self {
        let new_table = self.table.put_pure(key, value);
        GForm {
            table: Arc::new(new_table),
            parent: self.parent.clone(),
        }
    }
    
    /// Get the immediate table (not including parents)
    pub fn table(&self) -> &Arc<GTable> {
        &self.table
    }
    
    /// Get the parent form
    pub fn parent(&self) -> Option<&Arc<GForm>> {
        self.parent.as_ref()
    }
}

impl Clone for GForm {
    fn clone(&self) -> Self {
        GForm {
            table: Arc::clone(&self.table),
            parent: self.parent.clone(),
        }
    }
}

impl Default for GForm {
    fn default() -> Self {
        Self::new()
    }
}

impl Object for GForm {
    fn as_float(&self) -> Result<f64> {
        // Forms cannot be converted to float
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Object(self.clone_object()))
    }
    
    fn type_name(&self) -> &'static str {
        "GForm"
    }
    
    fn clone_object(&self) -> Rc<dyn Object> {
        Rc::new(self.clone())
    }
    
    fn is_zero(&self) -> bool {
        self.table.is_empty() && self.parent.is_none()
    }
}

impl std::fmt::Display for GForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GForm{{")?;
        // For simplicity, just show immediate table
        // In a full implementation, we'd show all inherited pairs
        write!(f, "table_size: {}", self.table.len())?;
        write!(f, "}}")
    }
}

/// Multiple inheritance support - linearize inheritance from multiple parents
/// This corresponds to the linearizeInheritance function in the C++ implementation
pub fn linearize_inheritance(parents: Vec<Rc<Form>>) -> Rc<Form> {
    if parents.is_empty() {
        return Rc::new(Form::new());
    }
    
    if parents.len() == 1 {
        return parents.into_iter().next().unwrap();
    }
    
    // For now, implement simple left-to-right linearization
    // A full implementation would use C3 linearization like Dylan
    let mut result = parents[0].clone();
    
    for parent in parents.iter().skip(1) {
        result = merge_forms(result, parent.clone());
    }
    
    result
}

/// Merge two forms using simple precedence rules
/// This is a simplified version of the merge algorithm in the C++ code
fn merge_forms(left: Rc<Form>, right: Rc<Form>) -> Rc<Form> {
    // Create a new form that inherits from both
    // For simplicity, we'll just use left as primary and right as secondary
    let mut merged_table = left.table().clone();
    
    // Add any keys from right that aren't in left
    for (key, value) in right.table().pairs() {
        if !merged_table.contains_key(&key) {
            merged_table = merged_table.put(key, value);
        }
    }
    
    Rc::new(Form::from_table(merged_table))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_basic_operations() {
        let table = Table::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        
        let key = Value::from("hello");
        let value = Value::from(42.0);
        
        let table2 = table.put(key.clone(), value.clone());
        assert_eq!(table2.len(), 1);
        assert!(!table2.is_empty());
        assert_eq!(table2.get(&key).unwrap(), &value);
    }
    
    #[test]
    fn test_form_inheritance() {
        // Create parent form
        let parent_table = Table::new()
            .put(Value::from("x"), Value::from(1.0))
            .put(Value::from("y"), Value::from(2.0));
        let parent = Rc::new(Form::from_table(parent_table));
        
        // Create child form
        let child_table = Table::new()
            .put(Value::from("z"), Value::from(3.0))
            .put(Value::from("x"), Value::from(10.0)); // Override parent's x
        let child = Form::with_parent(child_table, parent);
        
        // Test lookups
        assert_eq!(child.get(&Value::from("x")).unwrap().as_float().unwrap(), 10.0); // Overridden
        assert_eq!(child.get(&Value::from("y")).unwrap().as_float().unwrap(), 2.0);  // Inherited
        assert_eq!(child.get(&Value::from("z")).unwrap().as_float().unwrap(), 3.0);  // Own
        assert!(child.get(&Value::from("missing")).is_none());
    }
    
    #[test]
    fn test_gform_thread_safety() {
        let gform = GForm::new();
        
        let key = Value::from("test");
        let value = Value::from(123.0);
        
        gform.put_impure(key.clone(), value.clone()).unwrap();
        assert_eq!(gform.get(&key).unwrap().as_float().unwrap(), 123.0);
        
        // Test pure operation
        let new_gform = gform.put_pure(Value::from("new"), Value::from(456.0));
        assert_eq!(new_gform.get(&Value::from("new")).unwrap().as_float().unwrap(), 456.0);
        assert_eq!(new_gform.get(&key).unwrap().as_float().unwrap(), 123.0); // Still has old value
    }
    
    #[test]
    fn test_form_display() {
        let table = Table::new()
            .put(Value::from("hello"), Value::from("world"))
            .put(Value::from("pi"), Value::from(3.14159));
        let form = Form::from_table(table);
        
        let display = format!("{}", form);
        assert!(display.contains("hello"));
        assert!(display.contains("world"));
        assert!(display.contains("pi"));
    }
    
    #[test]
    fn test_linearize_inheritance() {
        let form1 = Rc::new(Form::from_table(
            Table::new().put(Value::from("a"), Value::from(1.0))
        ));
        let form2 = Rc::new(Form::from_table(
            Table::new().put(Value::from("b"), Value::from(2.0))
        ));
        let form3 = Rc::new(Form::from_table(
            Table::new().put(Value::from("c"), Value::from(3.0))
        ));
        
        let merged = linearize_inheritance(vec![form1, form2, form3]);
        
        // Should have all keys
        assert!(merged.contains_key(&Value::from("a")));
        assert!(merged.contains_key(&Value::from("b")));
        assert!(merged.contains_key(&Value::from("c")));
    }
    
    #[test]
    fn test_must_get_error() {
        let form = Form::new();
        let result = form.must_get(&Value::from("missing"));
        assert!(matches!(result, Err(SapfError::NotFound)));
    }
    
    #[test]
    fn test_form_zero_detection() {
        let empty_form = Form::new();
        assert!(empty_form.is_zero());
        
        let non_empty_form = Form::new().put(Value::from("x"), Value::from(1.0));
        assert!(!non_empty_form.is_zero());
    }
    
    #[test]
    fn test_hash_collision_handling() {
        let table = Table::new();
        
        // Test with different types that might hash similarly
        let key1 = Value::from(42.0);
        let key2 = Value::from("42");
        let value1 = Value::from("number");
        let value2 = Value::from("string");
        
        let table = table.put(key1.clone(), value1.clone())
                         .put(key2.clone(), value2.clone());
        
        assert_eq!(table.get(&key1).unwrap().as_object().unwrap().type_name(), "String");
        assert_eq!(table.get(&key2).unwrap().as_object().unwrap().type_name(), "String");
        assert_eq!(table.len(), 2);
    }
}
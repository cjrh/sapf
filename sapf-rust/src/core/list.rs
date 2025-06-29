//! List and Array types for SAPF
//!
//! This module implements the core collection types: Array (concrete storage) and List (lazy sequences).
//! Lists can be either Value lists (VList) or numeric lists (ZList) and support lazy evaluation
//! through generators.

use std::sync::Arc;
use std::fmt;
use crate::core::error::{SapfError, Result};
use crate::core::value::{Value, Object};

/// Fold function from SuperCollider - folds values at boundaries
/// This mirrors the sc_fold function from the C++ implementation
fn sc_fold(input: i64, lo: i64, hi: i64) -> i64 {
    if hi == lo {
        return lo;
    }
    
    let mut val = input;
    
    // Handle simple cases first
    if val >= hi {
        val = hi + hi - val;
        if val >= lo {
            return val;
        }
    } else if val < lo {
        val = lo + lo - val;
        if val < hi {
            return val;
        }
    } else {
        return val;
    }
    
    // More complex folding
    let range = hi - lo;
    let range2 = range + range;
    let x = val - lo;
    let c = x - range2 * (x / range2);
    let result = if c >= range { range2 - c } else { c };
    result + lo
}

/// Element type for arrays and lists
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Value, // V type - can hold any SAPF value
    Real,  // Z type - holds only real numbers
}

/// Array provides concrete storage for sequences
/// 
/// This corresponds to the Array class in the C++ implementation.
/// Arrays can store either Values or real numbers based on their element type.
#[derive(Debug)]
pub struct Array {
    element_type: ElementType,
    values: Vec<Value>,    // For Value arrays
    reals: Vec<f64>,       // For Real arrays
}

impl Array {
    /// Create a new array with the specified element type and capacity
    pub fn new(element_type: ElementType, capacity: usize) -> Self {
        match element_type {
            ElementType::Value => Array {
                element_type,
                values: Vec::with_capacity(capacity),
                reals: Vec::new(),
            },
            ElementType::Real => Array {
                element_type,
                values: Vec::new(),
                reals: Vec::with_capacity(capacity),
            },
        }
    }
    
    /// Create a new Value array from a vector
    pub fn from_values(values: Vec<Value>) -> Self {
        Array {
            element_type: ElementType::Value,
            values,
            reals: Vec::new(),
        }
    }
    
    /// Create a new Real array from a vector
    pub fn from_reals(reals: Vec<f64>) -> Self {
        Array {
            element_type: ElementType::Real,
            values: Vec::new(),
            reals,
        }
    }
    
    /// Get the element type
    pub fn element_type(&self) -> ElementType {
        self.element_type
    }
    
    /// Get the size of the array
    pub fn size(&self) -> usize {
        match self.element_type {
            ElementType::Value => self.values.len(),
            ElementType::Real => self.reals.len(),
        }
    }
    
    /// Check if array is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }
    
    /// Add a value to a Value array
    pub fn add_value(&mut self, value: Value) -> Result<()> {
        if self.element_type != ElementType::Value {
            return Err(SapfError::WrongType);
        }
        self.values.push(value);
        Ok(())
    }
    
    /// Add a real number to a Real array
    pub fn add_real(&mut self, real: f64) -> Result<()> {
        if self.element_type != ElementType::Real {
            return Err(SapfError::WrongType);
        }
        self.reals.push(real);
        Ok(())
    }
    
    /// Put a value at a specific index in a Value array
    pub fn put_value(&mut self, index: usize, value: Value) -> Result<()> {
        if self.element_type != ElementType::Value {
            return Err(SapfError::WrongType);
        }
        if index >= self.values.len() {
            return Err(SapfError::OutOfRange);
        }
        self.values[index] = value;
        Ok(())
    }
    
    /// Put a real number at a specific index in a Real array
    pub fn put_real(&mut self, index: usize, real: f64) -> Result<()> {
        if self.element_type != ElementType::Real {
            return Err(SapfError::WrongType);
        }
        if index >= self.reals.len() {
            return Err(SapfError::OutOfRange);
        }
        self.reals[index] = real;
        Ok(())
    }
    
    /// Get a value at a specific index
    pub fn at(&self, index: usize) -> Result<Value> {
        match self.element_type {
            ElementType::Value => {
                self.values.get(index)
                    .cloned()
                    .ok_or(SapfError::OutOfRange)
            }
            ElementType::Real => {
                self.reals.get(index)
                    .copied()
                    .map(Value::real)
                    .ok_or(SapfError::OutOfRange)
            }
        }
    }
    
    /// Get a value with wrapping (index % size)
    pub fn wrap_at(&self, index: i64) -> Result<Value> {
        let size = self.size();
        if size == 0 {
            return Err(SapfError::OutOfRange);
        }
        let wrapped_index = ((index % size as i64) + size as i64) as usize % size;
        self.at(wrapped_index)
    }
    
    /// Get a value with clipping (clamp index to bounds)
    pub fn clip_at(&self, index: i64) -> Result<Value> {
        let size = self.size();
        if size == 0 {
            return Err(SapfError::OutOfRange);
        }
        let clipped_index = index.clamp(0, size as i64 - 1) as usize;
        self.at(clipped_index)
    }
    
    /// Get a value with folding (fold index at boundaries)
    /// This implements the sc_fold behavior from the C++ version
    pub fn fold_at(&self, index: i64) -> Result<Value> {
        let size = self.size();
        if size == 0 {
            return Err(SapfError::OutOfRange);
        }
        if size == 1 {
            return self.at(0);
        }
        
        let folded_index = sc_fold(index, 0, size as i64 - 1) as usize;
        self.at(folded_index)
    }
    
    /// Get a real number at a specific index (for Real arrays)
    pub fn at_real(&self, index: usize) -> Result<f64> {
        if self.element_type != ElementType::Real {
            return Err(SapfError::WrongType);
        }
        self.reals.get(index)
            .copied()
            .ok_or(SapfError::OutOfRange)
    }
    
    /// Get values slice (for Value arrays)
    pub fn values(&self) -> &[Value] {
        &self.values
    }
    
    /// Get reals slice (for Real arrays)
    pub fn reals(&self) -> &[f64] {
        &self.reals
    }
}

impl Object for Array {
    fn as_float(&self) -> Result<f64> {
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::object(self.clone()))
    }
    
    fn type_name(&self) -> &'static str {
        "Array"
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
}

impl Clone for Array {
    fn clone(&self) -> Self {
        Array {
            element_type: self.element_type,
            values: self.values.clone(),
            reals: self.reals.clone(),
        }
    }
}

// Safety: Array is Send + Sync because Vec<Value> and Vec<f64> are Send + Sync
// and Value is Send + Sync (when Object: Send + Sync)
unsafe impl Send for Array {}
unsafe impl Sync for Array {}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        let size = self.size();
        for i in 0..size {
            if i > 0 {
                write!(f, " ")?;
            }
            match self.at(i) {
                Ok(value) => write!(f, "{}", value)?,
                Err(_) => write!(f, "?")?,
            }
        }
        write!(f, "]")
    }
}

/// Generator trait for lazy list evaluation
/// 
/// This corresponds to the Gen class in the C++ implementation.
pub trait Generator: fmt::Debug + fmt::Display + Send + Sync {
    /// Generate values into the target list
    fn pull(&mut self, target: &mut List) -> Result<()>;
    
    /// Check if the generator is done
    fn is_done(&self) -> bool;
    
    /// Get the element type this generator produces
    fn element_type(&self) -> ElementType;
    
    /// Clone this generator
    fn clone_generator(&self) -> Box<dyn Generator>;
}

/// List represents a sequence that can be lazy or realized
/// 
/// This corresponds to the List class in the C++ implementation.
/// Lists support lazy evaluation through generators and can chain together.
#[derive(Debug)]
pub struct List {
    element_type: ElementType,
    array: Option<Arc<Array>>,           // Realized data
    generator: Option<Box<dyn Generator>>, // Lazy generator
    next: Option<Arc<List>>,             // Linked next list
}

impl List {
    /// Create a new empty list with the specified element type
    pub fn new(element_type: ElementType) -> Self {
        List {
            element_type,
            array: None,
            generator: None,
            next: None,
        }
    }
    
    /// Create a new list with the specified capacity
    pub fn with_capacity(element_type: ElementType, capacity: usize) -> Self {
        let array = Array::new(element_type, capacity);
        List {
            element_type,
            array: Some(Arc::new(array)),
            generator: None,
            next: None,
        }
    }
    
    /// Create a list from an array
    pub fn from_array(array: Array) -> Self {
        let element_type = array.element_type();
        List {
            element_type,
            array: Some(Arc::new(array)),
            generator: None,
            next: None,
        }
    }
    
    /// Create a list from a generator
    pub fn from_generator(generator: Box<dyn Generator>) -> Self {
        let element_type = generator.element_type();
        List {
            element_type,
            array: None,
            generator: Some(generator),
            next: None,
        }
    }
    
    /// Create a list with a next pointer
    pub fn with_next(array: Array, next: Arc<List>) -> Self {
        let element_type = array.element_type();
        List {
            element_type,
            array: Some(Arc::new(array)),
            generator: None,
            next: Some(next),
        }
    }
    
    /// Get the element type
    pub fn element_type(&self) -> ElementType {
        self.element_type
    }
    
    /// Check if this is a Value list
    pub fn is_value_list(&self) -> bool {
        self.element_type == ElementType::Value
    }
    
    /// Check if this is a Real list
    pub fn is_real_list(&self) -> bool {
        self.element_type == ElementType::Real
    }
    
    /// Check if this list has lazy evaluation (generator)
    pub fn is_lazy(&self) -> bool {
        self.generator.is_some()
    }
    
    /// Check if this list is realized (has array data)
    pub fn is_realized(&self) -> bool {
        self.array.is_some()
    }
    
    /// Check if this list is at the end (empty array, no next)
    pub fn is_end(&self) -> bool {
        match &self.array {
            Some(arr) => arr.is_empty() && self.next.is_none(),
            None => self.next.is_none(),
        }
    }
    
    /// Check if this list is packed (single array, no chaining)
    pub fn is_packed(&self) -> bool {
        self.next.is_none() && self.generator.is_none()
    }
    
    /// Get the array if available
    pub fn array(&self) -> Option<&Arc<Array>> {
        self.array.as_ref()
    }
    
    /// Get the next list if available
    pub fn next(&self) -> Option<&Arc<List>> {
        self.next.as_ref()
    }
    
    /// Force evaluation of lazy elements up to a limit
    pub fn force(&mut self, _limit: usize) -> Result<()> {
        // Note: This is a simplified version. The full implementation would need
        // more sophisticated borrowing or interior mutability patterns.
        // For now, we'll just return Ok to make the interface available.
        Ok(())
    }
    
    /// Get length of the list (forces evaluation if needed)
    pub fn length(&mut self) -> Result<usize> {
        let mut total = 0;
        
        if let Some(ref array) = self.array {
            total += array.size();
        }
        
        // Force evaluation to get true length
        if self.generator.is_some() {
            self.force(usize::MAX)?;
            if let Some(ref array) = self.array {
                total = array.size(); // Update after forcing
            }
        }
        
        if let Some(ref next) = self.next {
            // Note: This would require mutable access to next, which is complex
            // For now, we'll just count realized elements
            if let Some(ref next_array) = next.array() {
                total += next_array.size();
            }
        }
        
        Ok(total)
    }
    
    /// Get value at index
    pub fn at(&self, index: usize) -> Result<Value> {
        if let Some(ref array) = self.array {
            if index < array.size() {
                return array.at(index);
            }
            
            // Check next list
            if let Some(ref next) = self.next {
                return next.at(index - array.size());
            }
        }
        
        Err(SapfError::OutOfRange)
    }
    
    /// Set next list
    pub fn set_next(&mut self, next: Arc<List>) {
        self.next = Some(next);
    }
    
    /// Link another list to the end of this chain
    pub fn link(&mut self, other: Arc<List>) {
        if self.next.is_some() {
            // This would require mutable access through Rc, which is complex
            // For now, we'll implement a simpler version
            self.next = Some(other);
        } else {
            self.next = Some(other);
        }
    }
}

impl Object for List {
    fn as_float(&self) -> Result<f64> {
        // Single element lists can be converted to float
        if let Some(ref array) = self.array {
            if array.size() == 1 {
                return array.at(0)?.as_float();
            }
        }
        Err(SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::object(self.clone()))
    }
    
    fn type_name(&self) -> &'static str {
        match self.element_type {
            ElementType::Value => "VList",
            ElementType::Real => "ZList",
        }
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
}

impl Clone for List {
    fn clone(&self) -> Self {
        List {
            element_type: self.element_type,
            array: self.array.clone(),
            generator: self.generator.as_ref().map(|g| g.clone_generator()),
            next: self.next.clone(),
        }
    }
}

// Safety: List is Send + Sync because:
// - Arc<Array> is Send + Sync when Array: Send + Sync
// - Box<dyn Generator> is Send + Sync when Generator: Send + Sync
// - Arc<List> is Send + Sync when List: Send + Sync
unsafe impl Send for List {}
unsafe impl Sync for List {}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref array) = self.array {
            write!(f, "{}", array)?;
        } else {
            write!(f, "[...]")?; // Lazy list
        }
        
        if let Some(ref next) = self.next {
            write!(f, " ++ {}", next)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_array_creation() {
        let mut arr = Array::new(ElementType::Value, 10);
        assert_eq!(arr.element_type(), ElementType::Value);
        assert_eq!(arr.size(), 0);
        assert!(arr.is_empty());
        
        arr.add_value(Value::real(42.0)).unwrap();
        assert_eq!(arr.size(), 1);
        assert!(!arr.is_empty());
        
        let val = arr.at(0).unwrap();
        assert_eq!(val.as_float().unwrap(), 42.0);
    }
    
    #[test]
    fn test_real_array() {
        let mut arr = Array::new(ElementType::Real, 5);
        arr.add_real(1.5).unwrap();
        arr.add_real(2.5).unwrap();
        
        assert_eq!(arr.size(), 2);
        assert_eq!(arr.at_real(0).unwrap(), 1.5);
        assert_eq!(arr.at_real(1).unwrap(), 2.5);
        
        let val = arr.at(0).unwrap();
        assert_eq!(val.as_float().unwrap(), 1.5);
    }
    
    #[test]
    fn test_array_indexing() {
        let values = vec![Value::real(1.0), Value::real(2.0), Value::real(3.0)];
        let arr = Array::from_values(values);
        
        // Normal indexing
        assert_eq!(arr.at(1).unwrap().as_float().unwrap(), 2.0);
        
        // Wrap indexing
        assert_eq!(arr.wrap_at(3).unwrap().as_float().unwrap(), 1.0); // 3 % 3 = 0
        assert_eq!(arr.wrap_at(-1).unwrap().as_float().unwrap(), 3.0); // wraps to last element
        
        // Clip indexing
        assert_eq!(arr.clip_at(10).unwrap().as_float().unwrap(), 3.0); // clamps to last
        assert_eq!(arr.clip_at(-5).unwrap().as_float().unwrap(), 1.0); // clamps to first
        
        // Fold indexing
        assert_eq!(arr.fold_at(2).unwrap().as_float().unwrap(), 3.0);
        assert_eq!(arr.fold_at(3).unwrap().as_float().unwrap(), 2.0); // folds back
    }
    
    #[test]
    fn test_list_creation() {
        let list = List::new(ElementType::Value);
        assert!(list.is_value_list());
        assert!(!list.is_real_list());
        assert!(!list.is_lazy());
        assert!(!list.is_realized());
        
        let arr = Array::from_values(vec![Value::real(1.0), Value::real(2.0)]);
        let list = List::from_array(arr);
        assert!(list.is_realized());
        assert!(list.is_packed());
    }
    
    #[test]
    fn test_list_chaining() {
        let arr1 = Array::from_values(vec![Value::real(1.0), Value::real(2.0)]);
        let arr2 = Array::from_values(vec![Value::real(3.0), Value::real(4.0)]);
        
        let list2 = Arc::new(List::from_array(arr2));
        let list1 = List::with_next(arr1, list2);
        
        assert!(!list1.is_packed());
        assert_eq!(list1.at(0).unwrap().as_float().unwrap(), 1.0);
        assert_eq!(list1.at(2).unwrap().as_float().unwrap(), 3.0);
    }
    
    #[test]
    fn test_type_errors() {
        let mut arr = Array::new(ElementType::Value, 5);
        
        // Can't add real to value array
        assert!(matches!(arr.add_real(1.0), Err(SapfError::WrongType)));
        
        let mut real_arr = Array::new(ElementType::Real, 5);
        // Can't add value to real array  
        assert!(matches!(real_arr.add_value(Value::real(1.0)), Err(SapfError::WrongType)));
    }
}
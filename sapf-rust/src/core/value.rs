//! Value type for SAPF
//! 
//! This module defines the central Value type that represents all values in the SAPF language.
//! The Value type is a tagged union that can hold either a real number (f64) or a reference
//! to an Object.

use std::sync::Arc;
use std::fmt;
use crate::core::error::{SapfError, Result};

/// Core trait for all SAPF objects
pub trait Object: fmt::Debug + fmt::Display + Send + Sync {
    /// Convert the object to a floating-point number
    fn as_float(&self) -> Result<f64>;
    
    /// Convert the object to an integer
    fn as_int(&self) -> Result<i64> {
        Ok(self.as_float()?.round() as i64)
    }
    
    /// Dereference the object (for references and such)
    fn deref(&self) -> Result<Value>;
    
    /// Get the type name for error messages
    fn type_name(&self) -> &'static str;
    
    /// Check if this object represents zero
    fn is_zero(&self) -> bool {
        false // Most objects are not zero
    }
    
    /// Check if this object is a function
    fn is_function(&self) -> bool {
        false
    }
    
    /// Check if this object is a primitive function
    fn is_primitive(&self) -> bool {
        false
    }
    
    /// Check if this object is a function definition
    fn is_function_def(&self) -> bool {
        false
    }
    
    /// Check if this object is a function or primitive (callable)
    fn is_function_or_primitive(&self) -> bool {
        self.is_function() || self.is_primitive()
    }
    
    /// Get help string for this object
    fn help(&self) -> Option<&str> {
        None
    }
    
    /// Clone this object
    fn clone_object(&self) -> Arc<dyn Object>;
    
    /// Get this object as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Apply a unary mathematical operation to this object
    fn unary_op(&self, _thread: &mut crate::vm::Thread, op: &dyn crate::core::math_ops::UnaryOp) -> Result<Value> {
        // Default implementation - try to convert to float and apply operation
        let float_val = self.as_float()?;
        let result = op.op(float_val);
        Ok(Value::Real(result))
    }
    
    /// Apply a binary mathematical operation to this object
    fn binary_op(&self, _thread: &mut crate::vm::Thread, op: &dyn crate::core::math_ops::BinaryOp, other: Value) -> Result<Value> {
        // Default implementation - try to convert both to float and apply operation
        let self_float = self.as_float()?;
        let other_float = other.as_float()?;
        let result = op.op(self_float, other_float);
        Ok(Value::Real(result))
    }
    
    /// Apply a binary mathematical operation with a real number as left operand
    fn binary_op_with_real(&self, _thread: &mut crate::vm::Thread, op: &dyn crate::core::math_ops::BinaryOp, real_val: f64) -> Result<Value> {
        // Default implementation - try to convert this to float and apply operation
        let self_float = self.as_float()?;
        let result = op.op(real_val, self_float);
        Ok(Value::Real(result))
    }
}

/// The central Value type in SAPF
/// 
/// A Value can be either a real number (f64) or a reference to an Object.
/// This corresponds to the V class in the C++ implementation.
#[derive(Debug, Clone)]
pub enum Value {
    /// A real number value
    Real(f64),
    /// A reference to an object
    Object(Arc<dyn Object>),
    /// The nil/null value
    Nil,
}

impl Value {
    /// Create a new real value
    pub fn real(f: f64) -> Self {
        Value::Real(f)
    }
    
    /// Create a new object value
    pub fn object<T: Object + 'static>(obj: T) -> Self {
        Value::Object(Arc::new(obj))
    }
    
    /// Create a new object value from Arc
    pub fn from_object(obj: Arc<dyn Object>) -> Self {
        Value::Object(obj)
    }
    
    /// Check if this value is a real number
    pub fn is_real(&self) -> bool {
        matches!(self, Value::Real(_))
    }
    
    /// Check if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
    
    /// Check if this value is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }
    
    /// Check if this value is zero
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Real(f) => *f == 0.0,
            Value::Object(obj) => obj.is_zero(),
            Value::Nil => true, // nil is considered zero/falsy
        }
    }
    
    /// Convert to floating-point number
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Real(f) => Ok(*f),
            Value::Object(obj) => obj.as_float(),
            Value::Nil => Ok(0.0), // nil converts to 0
        }
    }
    
    /// Convert to integer
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Real(f) => Ok(f.round() as i64),
            Value::Object(obj) => obj.as_int(),
            Value::Nil => Ok(0), // nil converts to 0
        }
    }
    
    /// Get as object, returning error if it's a real number
    pub fn as_object(&self) -> Result<&Arc<dyn Object>> {
        match self {
            Value::Real(_) => Err(SapfError::WrongType),
            Value::Object(obj) => Ok(obj),
            Value::Nil => Err(SapfError::WrongType),
        }
    }
    
    /// Dereference the value
    pub fn deref(&self) -> Result<Value> {
        match self {
            Value::Real(_) => Ok(self.clone()),
            Value::Object(obj) => obj.deref(),
            Value::Nil => Ok(self.clone()),
        }
    }
    
    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Real(_) => "Real",
            Value::Object(obj) => obj.type_name(),
            Value::Nil => "Nil",
        }
    }
    
    /// Convert to bool (for conditional operations)
    pub fn is_truthy(&self) -> bool {
        !self.is_zero()
    }
    
    /// Check if this value is a function
    pub fn is_function(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Object(obj) => obj.is_function(),
            Value::Nil => false,
        }
    }
    
    /// Check if this value is a primitive function
    pub fn is_primitive(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Object(obj) => obj.is_primitive(),
            Value::Nil => false,
        }
    }
    
    /// Check if this value is a function definition
    pub fn is_function_def(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Object(obj) => obj.is_function_def(),
            Value::Nil => false,
        }
    }
    
    /// Check if this value is callable (function or primitive)
    pub fn is_callable(&self) -> bool {
        match self {
            Value::Real(_) => false,
            Value::Object(obj) => obj.is_function_or_primitive(),
            Value::Nil => false,
        }
    }
    
    /// Apply this value as a function
    pub fn apply(&self, thread: &mut crate::vm::Thread) -> Result<()> {
        match self {
            Value::Real(_) => Err(SapfError::WrongType),
            Value::Object(obj) => {
                if let Ok(function) = obj.as_any().downcast_ref::<crate::core::function::Function>() {
                    function.apply(thread)
                } else if let Ok(primitive) = obj.as_any().downcast_ref::<crate::core::function::Primitive>() {
                    primitive.apply(thread)
                } else {
                    Err(SapfError::WrongType)
                }
            }
            Value::Nil => Err(SapfError::WrongType),
        }
    }
    
    /// Compare two values
    pub fn compare(&self, other: &Value) -> Result<std::cmp::Ordering> {
        use std::cmp::Ordering;
        
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => Ok(a.partial_cmp(b).unwrap_or(Ordering::Equal)),
            (Value::Nil, Value::Nil) => Ok(Ordering::Equal),
            (Value::Nil, _) => Ok(Ordering::Less),
            (_, Value::Nil) => Ok(Ordering::Greater),
            (Value::Real(_), Value::Object(_)) => Ok(Ordering::Less),
            (Value::Object(_), Value::Real(_)) => Ok(Ordering::Greater),
            (Value::Object(a), Value::Object(b)) => {
                // For objects, compare string representations
                let a_str = format!("{}", a);
                let b_str = format!("{}", b);
                Ok(a_str.cmp(&b_str))
            }
        }
    }
    
    /// Check if two values are equal
    pub fn equals(&self, other: &Value) -> Result<bool> {
        Ok(self.compare(other)? == std::cmp::Ordering::Equal)
    }
    
    /// Get a printable string representation
    pub fn print(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Real(val) => {
                if val.fract() == 0.0 && val.abs() < 1e15 {
                    // Display as integer if it's a whole number
                    write!(f, "{}", *val as i64)
                } else {
                    write!(f, "{}", val)
                }
            }
            Value::Object(obj) => write!(f, "{}", obj),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Real(f)
    }
}

// Safety: Value is Send + Sync because:
// - f64 is Send + Sync
// - Arc<dyn Object> is Send + Sync when Object: Send + Sync
unsafe impl Send for Value {}
unsafe impl Sync for Value {}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Real(f as f64)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Real(i as f64)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Real(i as f64)
    }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self {
        Value::Real(i as f64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Real(if b { 1.0 } else { 0.0 })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Real(a), Value::Real(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => {
                // For objects, we compare their string representations
                // This is a simplification - ideally we'd have proper equality
                format!("{}", a) == format!("{}", b)
            }
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }
}

/// Helper trait for types that can be converted to Value
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl<T: Into<Value>> IntoValue for T {
    fn into_value(self) -> Value {
        self.into()
    }
}

/// String object implementation
#[derive(Debug, Clone, PartialEq)]
pub struct StringObject {
    value: String,
    hash: i32,
}

impl StringObject {
    pub fn new(s: String) -> Self {
        let hash = crate::core::hash::hash_str(&s);
        StringObject { value: s, hash }
    }
    
    pub fn from_str(s: &str) -> Self {
        Self::new(s.to_string())
    }
    
    pub fn as_str(&self) -> &str {
        &self.value
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn len(&self) -> usize {
        self.value.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
    
    pub fn hash(&self) -> i32 {
        self.hash
    }
    
    /// Concatenate with another string
    pub fn concat(&self, other: &str) -> StringObject {
        let new_value = format!("{}{}", self.value, other);
        StringObject::new(new_value)
    }
    
    /// Compare with another string
    pub fn compare(&self, other: &StringObject) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl Object for StringObject {
    fn as_float(&self) -> Result<f64> {
        self.value.parse().map_err(|_| SapfError::WrongType)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Object(self.clone_object()))
    }
    
    fn type_name(&self) -> &'static str {
        "String"
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn is_zero(&self) -> bool {
        self.value.is_empty()
    }
}

impl fmt::Display for StringObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl Eq for StringObject {}

impl std::hash::Hash for StringObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::object(StringObject::new(s))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::object(StringObject::from_str(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_real_values() {
        let v = Value::real(42.0);
        assert!(v.is_real());
        assert!(!v.is_object());
        assert_eq!(v.as_float().unwrap(), 42.0);
        assert_eq!(v.as_int().unwrap(), 42);
        assert_eq!(v.type_name(), "Real");
    }
    
    #[test]
    fn test_zero_detection() {
        assert!(Value::real(0.0).is_zero());
        assert!(!Value::real(1.0).is_zero());
        assert!(!Value::real(-1.0).is_zero());
    }
    
    #[test]
    fn test_conversions() {
        let v1: Value = 42.0.into();
        let v2: Value = 42i32.into();
        let v3: Value = true.into();
        let v4: Value = false.into();
        
        assert_eq!(v1.as_float().unwrap(), 42.0);
        assert_eq!(v2.as_float().unwrap(), 42.0);
        assert_eq!(v3.as_float().unwrap(), 1.0);
        assert_eq!(v4.as_float().unwrap(), 0.0);
    }
    
    #[test]
    fn test_string_object() {
        let str_obj = StringObject::from_str("42.5");
        let v = Value::object(str_obj);
        
        assert!(v.is_object());
        assert!(!v.is_real());
        assert_eq!(v.as_float().unwrap(), 42.5);
        assert_eq!(v.type_name(), "String");
    }
    
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Value::real(42.0)), "42");
        assert_eq!(format!("{}", Value::real(42.5)), "42.5");
        
        // Very small numbers may be displayed differently by Rust
        let small_val = format!("{}", Value::real(1e-10));
        assert!(small_val == "1e-10" || small_val == "0.0000000001");
        
        let str_obj = StringObject::from_str("hello");
        let v = Value::object(str_obj);
        assert_eq!(format!("{}", v), "\"hello\"");
    }
    
    #[test]
    fn test_truthiness() {
        assert!(!Value::real(0.0).is_truthy());
        assert!(Value::real(1.0).is_truthy());
        assert!(Value::real(-1.0).is_truthy());
        
        let str_obj = StringObject::from_str("hello");
        let v = Value::object(str_obj);
        assert!(v.is_truthy()); // Non-zero string object is truthy
    }
    
    #[test]
    fn test_deref() {
        let v = Value::real(42.0);
        let deref_v = v.deref().unwrap();
        assert_eq!(deref_v.as_float().unwrap(), 42.0);
        
        let str_obj = StringObject::from_str("hello");
        let v = Value::object(str_obj);
        let deref_v = v.deref().unwrap();
        assert!(deref_v.is_object());
    }
    
    #[test]
    fn test_type_errors() {
        let v = Value::real(42.0);
        assert!(matches!(v.as_object(), Err(SapfError::WrongType)));
        
        let bad_str = StringObject::from_str("not_a_number");
        let v = Value::object(bad_str);
        assert!(matches!(v.as_float(), Err(SapfError::WrongType)));
    }
    
    #[test]
    fn test_string_operations() {
        let s1 = StringObject::from_str("hello");
        let s2 = StringObject::from_str("world");
        let s3 = StringObject::from_str("hello");
        
        // Test equality
        assert_eq!(s1, s3);
        assert_ne!(s1, s2);
        
        // Test hash consistency
        assert_eq!(s1.hash(), s3.hash());
        
        // Test concatenation
        let concat = s1.concat(" world");
        assert_eq!(concat.as_str(), "hello world");
        
        // Test comparison
        assert_eq!(s1.compare(&s2), std::cmp::Ordering::Less);
        assert_eq!(s2.compare(&s1), std::cmp::Ordering::Greater);
        assert_eq!(s1.compare(&s3), std::cmp::Ordering::Equal);
    }
    
    #[test]
    fn test_string_zero() {
        let empty = StringObject::from_str("");
        let nonempty = StringObject::from_str("hello");
        
        assert!(empty.is_zero());
        assert!(!nonempty.is_zero());
        
        let empty_val = Value::object(empty);
        let nonempty_val = Value::object(nonempty);
        
        assert!(empty_val.is_zero());
        assert!(!nonempty_val.is_zero());
    }
    
    #[test]
    fn test_string_conversions() {
        // Test From implementations
        let v1: Value = "hello".into();
        let v2: Value = "world".to_string().into();
        
        assert!(v1.is_object());
        assert!(v2.is_object());
        
        assert_eq!(v1.type_name(), "String");
        assert_eq!(v2.type_name(), "String");
    }
}
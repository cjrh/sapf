//! Value type for SAPF
//! 
//! This module defines the central Value type that represents all values in the SAPF language.
//! The Value type is a tagged union that can hold either a real number (f64) or a reference
//! to an Object.

use std::rc::Rc;
use std::fmt;
use crate::core::error::{SapfError, Result};

/// Core trait for all SAPF objects
pub trait Object: fmt::Debug + fmt::Display {
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
    
    /// Clone this object
    fn clone_object(&self) -> Rc<dyn Object>;
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
    Object(Rc<dyn Object>),
}

impl Value {
    /// Create a new real value
    pub fn real(f: f64) -> Self {
        Value::Real(f)
    }
    
    /// Create a new object value
    pub fn object<T: Object + 'static>(obj: T) -> Self {
        Value::Object(Rc::new(obj))
    }
    
    /// Create a new object value from Rc
    pub fn from_object(obj: Rc<dyn Object>) -> Self {
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
    
    /// Check if this value is zero
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Real(f) => *f == 0.0,
            Value::Object(obj) => obj.is_zero(),
        }
    }
    
    /// Convert to floating-point number
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Real(f) => Ok(*f),
            Value::Object(obj) => obj.as_float(),
        }
    }
    
    /// Convert to integer
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Real(f) => Ok(f.round() as i64),
            Value::Object(obj) => obj.as_int(),
        }
    }
    
    /// Get as object, returning error if it's a real number
    pub fn as_object(&self) -> Result<&Rc<dyn Object>> {
        match self {
            Value::Real(_) => Err(SapfError::WrongType),
            Value::Object(obj) => Ok(obj),
        }
    }
    
    /// Dereference the value
    pub fn deref(&self) -> Result<Value> {
        match self {
            Value::Real(_) => Ok(self.clone()),
            Value::Object(obj) => obj.deref(),
        }
    }
    
    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Real(_) => "Real",
            Value::Object(obj) => obj.type_name(),
        }
    }
    
    /// Convert to bool (for conditional operations)
    pub fn is_truthy(&self) -> bool {
        !self.is_zero()
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
        }
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Real(f)
    }
}

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

/// Helper trait for types that can be converted to Value
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl<T: Into<Value>> IntoValue for T {
    fn into_value(self) -> Value {
        self.into()
    }
}

/// Basic String object implementation for testing
#[derive(Debug, Clone)]
pub struct StringObject {
    value: String,
}

impl StringObject {
    pub fn new(s: String) -> Self {
        StringObject { value: s }
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
    
    fn clone_object(&self) -> Rc<dyn Object> {
        Rc::new(self.clone())
    }
}

impl fmt::Display for StringObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
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
        let str_obj = StringObject::new("42.5".to_string());
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
        
        let str_obj = StringObject::new("hello".to_string());
        let v = Value::object(str_obj);
        assert_eq!(format!("{}", v), "\"hello\"");
    }
    
    #[test]
    fn test_truthiness() {
        assert!(!Value::real(0.0).is_truthy());
        assert!(Value::real(1.0).is_truthy());
        assert!(Value::real(-1.0).is_truthy());
        
        let str_obj = StringObject::new("hello".to_string());
        let v = Value::object(str_obj);
        assert!(v.is_truthy()); // Non-zero string object is truthy
    }
    
    #[test]
    fn test_deref() {
        let v = Value::real(42.0);
        let deref_v = v.deref().unwrap();
        assert_eq!(deref_v.as_float().unwrap(), 42.0);
        
        let str_obj = StringObject::new("hello".to_string());
        let v = Value::object(str_obj);
        let deref_v = v.deref().unwrap();
        assert!(deref_v.is_object());
    }
    
    #[test]
    fn test_type_errors() {
        let v = Value::real(42.0);
        assert!(matches!(v.as_object(), Err(SapfError::WrongType)));
        
        let bad_str = StringObject::new("not_a_number".to_string());
        let v = Value::object(bad_str);
        assert!(matches!(v.as_float(), Err(SapfError::WrongType)));
    }
}
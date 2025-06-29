//! Reference types for SAPF
//!
//! This module implements mutable containers for SAPF values.
//! Ref can hold any Value, while ZRef specifically holds real numbers.

use std::sync::{Arc, Mutex};
use std::fmt;
use crate::core::error::{SapfError, Result};
use crate::core::value::{Value, Object};

/// Mutable reference to a SAPF value
/// 
/// This corresponds to the Ref class in the C++ implementation.
/// It provides thread-safe mutable access to a Value.
#[derive(Debug)]
pub struct Ref {
    inner: Arc<Mutex<Value>>,
}

impl Ref {
    /// Create a new Ref containing the given value
    pub fn new(value: Value) -> Self {
        Ref {
            inner: Arc::new(Mutex::new(value)),
        }
    }
    
    /// Set the value in this reference
    pub fn set(&self, value: Value) -> Result<()> {
        match self.inner.lock() {
            Ok(mut guard) => {
                *guard = value;
                Ok(())
            }
            Err(_) => Err(SapfError::InternalError), // Mutex poisoned
        }
    }
    
    /// Get the current value (dereference)
    pub fn get(&self) -> Result<Value> {
        match self.inner.lock() {
            Ok(guard) => Ok(guard.clone()),
            Err(_) => Err(SapfError::InternalError), // Mutex poisoned
        }
    }
    
    /// Update the value using a closure
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Value),
    {
        match self.inner.lock() {
            Ok(mut guard) => {
                f(&mut *guard);
                Ok(())
            }
            Err(_) => Err(SapfError::InternalError),
        }
    }
    
    /// Apply a transformation and return the result
    pub fn with<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Value) -> R,
    {
        match self.inner.lock() {
            Ok(guard) => Ok(f(&*guard)),
            Err(_) => Err(SapfError::InternalError),
        }
    }
}

impl Object for Ref {
    fn as_float(&self) -> Result<f64> {
        match self.get() {
            Ok(value) => value.as_float(),
            Err(e) => Err(e),
        }
    }
    
    fn deref(&self) -> Result<Value> {
        self.get()
    }
    
    fn type_name(&self) -> &'static str {
        "Ref"
    }
    
    fn clone_object(&self) -> std::rc::Rc<dyn Object> {
        std::rc::Rc::new(self.clone())
    }
}

impl Clone for Ref {
    fn clone(&self) -> Self {
        // Clone the Arc, sharing the same mutex
        Ref {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get() {
            Ok(value) => write!(f, "&{}", value),
            Err(_) => write!(f, "&<error>"),
        }
    }
}

impl PartialEq for Ref {
    fn eq(&self, other: &Self) -> bool {
        // Two refs are equal if they point to the same data
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

/// Mutable reference to a real number
/// 
/// This corresponds to the ZRef class in the C++ implementation.
/// It provides thread-safe mutable access to a real number.
#[derive(Debug)]
pub struct ZRef {
    inner: Arc<Mutex<f64>>,
}

impl ZRef {
    /// Create a new ZRef containing the given real number
    pub fn new(value: f64) -> Self {
        ZRef {
            inner: Arc::new(Mutex::new(value)),
        }
    }
    
    /// Set the value in this reference
    pub fn set(&self, value: f64) -> Result<()> {
        match self.inner.lock() {
            Ok(mut guard) => {
                *guard = value;
                Ok(())
            }
            Err(_) => Err(SapfError::InternalError),
        }
    }
    
    /// Get the current value
    pub fn get(&self) -> Result<f64> {
        match self.inner.lock() {
            Ok(guard) => Ok(*guard),
            Err(_) => Err(SapfError::InternalError),
        }
    }
    
    /// Update the value using a closure
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut f64),
    {
        match self.inner.lock() {
            Ok(mut guard) => {
                f(&mut *guard);
                Ok(())
            }
            Err(_) => Err(SapfError::InternalError),
        }
    }
    
    /// Apply a transformation and return the result
    pub fn with<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&f64) -> R,
    {
        match self.inner.lock() {
            Ok(guard) => Ok(f(&*guard)),
            Err(_) => Err(SapfError::InternalError),
        }
    }
}

impl Object for ZRef {
    fn as_float(&self) -> Result<f64> {
        self.get()
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::real(self.get()?))
    }
    
    fn type_name(&self) -> &'static str {
        "ZRef"
    }
    
    fn clone_object(&self) -> std::rc::Rc<dyn Object> {
        std::rc::Rc::new(self.clone())
    }
}

impl Clone for ZRef {
    fn clone(&self) -> Self {
        // Clone the Arc, sharing the same mutex
        ZRef {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Display for ZRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get() {
            Ok(value) => write!(f, "&{}", value),
            Err(_) => write!(f, "&<error>"),
        }
    }
}

impl PartialEq for ZRef {
    fn eq(&self, other: &Self) -> bool {
        // Two zrefs are equal if they point to the same data
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ref_creation_and_access() {
        let r = Ref::new(Value::real(42.0));
        
        let value = r.get().unwrap();
        assert_eq!(value.as_float().unwrap(), 42.0);
        
        // Test as_float through Object trait
        assert_eq!(r.as_float().unwrap(), 42.0);
    }
    
    #[test]
    fn test_ref_set() {
        let r = Ref::new(Value::real(42.0));
        
        r.set(Value::real(24.0)).unwrap();
        assert_eq!(r.get().unwrap().as_float().unwrap(), 24.0);
    }
    
    #[test]
    fn test_ref_update() {
        let r = Ref::new(Value::real(10.0));
        
        r.update(|v| {
            if let Ok(val) = v.as_float() {
                *v = Value::real(val * 2.0);
            }
        }).unwrap();
        
        assert_eq!(r.get().unwrap().as_float().unwrap(), 20.0);
    }
    
    #[test]
    fn test_ref_with() {
        let r = Ref::new(Value::real(5.0));
        
        let doubled = r.with(|v| v.as_float().unwrap() * 2.0).unwrap();
        assert_eq!(doubled, 10.0);
        
        // Original value unchanged
        assert_eq!(r.get().unwrap().as_float().unwrap(), 5.0);
    }
    
    #[test]
    fn test_ref_clone() {
        let r1 = Ref::new(Value::real(42.0));
        let r2 = r1.clone();
        
        // They should point to the same data
        assert_eq!(r1, r2);
        
        // Changing one affects the other
        r1.set(Value::real(24.0)).unwrap();
        assert_eq!(r2.get().unwrap().as_float().unwrap(), 24.0);
    }
    
    #[test]
    fn test_zref_creation_and_access() {
        let zr = ZRef::new(42.0);
        
        assert_eq!(zr.get().unwrap(), 42.0);
        assert_eq!(zr.as_float().unwrap(), 42.0);
        
        let value = zr.deref().unwrap();
        assert_eq!(value.as_float().unwrap(), 42.0);
    }
    
    #[test]
    fn test_zref_set() {
        let zr = ZRef::new(42.0);
        
        zr.set(24.0).unwrap();
        assert_eq!(zr.get().unwrap(), 24.0);
    }
    
    #[test]
    fn test_zref_update() {
        let zr = ZRef::new(10.0);
        
        zr.update(|v| *v *= 2.0).unwrap();
        assert_eq!(zr.get().unwrap(), 20.0);
    }
    
    #[test]
    fn test_zref_with() {
        let zr = ZRef::new(5.0);
        
        let doubled = zr.with(|v| *v * 2.0).unwrap();
        assert_eq!(doubled, 10.0);
        
        // Original value unchanged
        assert_eq!(zr.get().unwrap(), 5.0);
    }
    
    #[test]
    fn test_zref_clone() {
        let zr1 = ZRef::new(42.0);
        let zr2 = zr1.clone();
        
        // They should point to the same data
        assert_eq!(zr1, zr2);
        
        // Changing one affects the other
        zr1.set(24.0).unwrap();
        assert_eq!(zr2.get().unwrap(), 24.0);
    }
    
    #[test]
    fn test_ref_display() {
        let r = Ref::new(Value::real(42.0));
        let display = format!("{}", r);
        assert!(display.starts_with("&42"));
        
        let zr = ZRef::new(42.0);
        let display = format!("{}", zr);
        assert!(display.starts_with("&42"));
    }
}
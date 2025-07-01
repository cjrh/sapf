//! Stream Operations - Implementation of SAPF list/stream operations
//!
//! This module implements the stream operations from StreamOps.cpp, providing
//! lazy evaluation, infinite sequences, and functional programming primitives
//! for the SAPF language.
//!
//! The implementation builds on the existing List lazy evaluation infrastructure
//! and follows the UGen pull() pattern for streaming data processing.

use crate::core::{
    error::SapfError,
    value::{Value, StringObject},
    list::{List, Array, ElementType},
    function::Primitive,
};
use crate::vm::thread::Thread;
use std::sync::Arc;

// Helper function to downcast Value to List
fn value_as_list(value: &Value) -> Option<&List> {
    if let Value::Object(obj) = value {
        obj.as_any().downcast_ref::<List>()
    } else {
        None
    }
}

// ============================================================================
// 8.1.1 Basic List Operations
// ============================================================================

/// Get the size/length of a sequence
/// Returns infinity for infinite sequences
pub fn size_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        if list.is_infinite() {
            thread.push(Value::Real(f64::INFINITY));
        } else {
            // Need to clone to get mutable access for length
            let mut list_clone = list.clone();
            let len = list_clone.length()? as f64;
            thread.push(Value::Real(len));
        }
    } else {
        // For non-lists, return 1 (single item)
        thread.push(Value::Real(1.0));
    }

    Ok(())
}

/// Get the rank (number of nested dimensions) of an object
pub fn rank_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let mut v = thread.pop_value()?;
    let mut rank = 0;

    while let Some(list) = value_as_list(&v) {
        rank += 1;
        // Get first element to determine nested structure
        match list.at(0) {
            Ok(first) => v = first,
            Err(_) => break, // Empty list or error
        }
    }

    thread.push(Value::Real(rank as f64));
    Ok(())
}

/// Get the shape (dimensions) of an object
pub fn shape_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let mut v = thread.pop_value()?;
    let mut shape_vec = Vec::new();

    while let Some(list) = value_as_list(&v) {
        let len = if list.is_infinite() {
            f64::INFINITY
        } else {
            let mut list_clone = list.clone();
            list_clone.length()? as f64
        };
        shape_vec.push(Value::Real(len));

        // Get first element to continue shape analysis
        match list.at(0) {
            Ok(first) => v = first,
            Err(_) => break, // Empty list or error
        }
    }

    // Return shape as a list of dimensions
    let shape_array = Array::from_values(shape_vec);
    let shape_list = List::from_array(shape_array);
    thread.push(Value::Object(Arc::new(shape_list)));
    Ok(())
}

/// Check if a sequence is finite
pub fn finite_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    let is_finite = if let Some(list) = value_as_list(&v) {
        !list.is_infinite()
    } else {
        true // Non-lists are considered finite
    };

    thread.push(Value::Real(if is_finite { 1.0 } else { 0.0 }));
    Ok(())
}

/// Check if a list is empty
pub fn empty_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    let is_empty = if let Some(list) = value_as_list(&v) {
        let mut list_clone = list.clone();
        list_clone.length().unwrap_or(0) == 0
    } else {
        false // Non-lists are not empty
    };

    thread.push(Value::Real(if is_empty { 1.0 } else { 0.0 }));
    Ok(())
}

/// Check if a list is non-empty
pub fn nonempty_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    let is_nonempty = if let Some(list) = value_as_list(&v) {
        let mut list_clone = list.clone();
        list_clone.length().unwrap_or(0) > 0
    } else {
        true // Non-lists are considered non-empty
    };

    thread.push(Value::Real(if is_nonempty { 1.0 } else { 0.0 }));
    Ok(())
}

/// Get the first item of a list (head)
pub fn head_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        match list.at(0) {
            Ok(first) => thread.push(first),
            Err(_) => return Err(SapfError::InvalidValue("head of empty list".to_string())),
        }
    } else {
        return Err(SapfError::WrongType);
    }

    Ok(())
}

/// Get the tail of a list (all items except the first)
pub fn tail_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        let mut list_clone = list.clone();
        let len = match list_clone.length() {
            Ok(l) => l,
            Err(_) => return Err(SapfError::InvalidValue("tail of empty list".to_string())),
        };

        if len == 0 {
            return Err(SapfError::InvalidValue("tail of empty list".to_string()));
        }

        // Create a new list with all elements except the first
        let mut tail_values = Vec::new();

        for i in 1..len {
            match list.at(i) {
                Ok(item) => tail_values.push(item),
                Err(_) => break,
            }
        }

        let tail_array = Array::from_values(tail_values);
        let tail_list = List::from_array(tail_array);
        thread.push(Value::Object(Arc::new(tail_list)));
    } else {
        return Err(SapfError::WrongType);
    }

    Ok(())
}

/// Check if a list is packed (stored contiguously)
pub fn packed_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    let is_packed = if let Some(list) = value_as_list(&v) {
        list.is_packed()
    } else {
        true // Non-lists are considered packed
    };

    thread.push(Value::Real(if is_packed { 1.0 } else { 0.0 }));
    Ok(())
}

/// Pack a list into contiguous storage
pub fn pack_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        // For now, just return the list as-is since lists are already packed
        // TODO: Implement actual packing if needed for lazy lists
        thread.push(v);
    } else {
        // Non-lists are returned as-is
        thread.push(v);
    }

    Ok(())
}

// ============================================================================
// 8.1.2 List Conversion Functions
// ============================================================================

/// Convert any value to a Value list (VList)
/// - ZList: Convert to VList element by element
/// - String: Convert each character to a Value
/// - Other: Return as-is
pub fn v_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        // Check if it's already a Value list
        if list.element_type() == ElementType::Value {
            thread.push(v);
        } else {
            // Convert ZList to VList - create new list with same values
            let mut list_clone = list.clone();
            let len = list_clone.length()?;
            let mut values = Vec::new();
            
            for i in 0..len {
                match list.at(i) {
                    Ok(item) => values.push(item),
                    Err(_) => break,
                }
            }
            
            let array = Array::from_values(values);
            let vlist = List::from_array(array);
            thread.push(Value::Object(Arc::new(vlist)));
        }
    } else if let Value::Object(obj) = &v {
        // Check if it's a string
        if let Some(string_obj) = obj.as_any().downcast_ref::<StringObject>() {
            // Convert string to VList
            let s = string_obj.value();
            let mut values = Vec::new();
            
            for ch in s.chars() {
                values.push(Value::Real(ch as u32 as f64));
            }
            
            let array = Array::from_values(values);
            let vlist = List::from_array(array);
            thread.push(Value::Object(Arc::new(vlist)));
        } else {
            // Return other objects as-is
            thread.push(v);
        }
    } else {
        // Return other values as-is
        thread.push(v);
    }

    Ok(())
}

/// Convert any value to a Real list (ZList)
/// - VList: Convert to ZList element by element (extract numeric values)
/// - String: Convert each character to a Real
/// - Other: Return as-is
pub fn z_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        // Check if it's already a Real list
        if list.element_type() == ElementType::Real {
            thread.push(v);
        } else {
            // Convert VList to ZList - extract numeric values
            let mut list_clone = list.clone();
            let len = list_clone.length()?;
            let mut reals = Vec::new();
            
            for i in 0..len {
                match list.at(i) {
                    Ok(item) => {
                        match item.as_float() {
                            Ok(f) => reals.push(f),
                            Err(_) => reals.push(0.0), // Non-numeric values become 0
                        }
                    }
                    Err(_) => break,
                }
            }
            
            let array = Array::from_reals(reals);
            let zlist = List::from_array(array);
            thread.push(Value::Object(Arc::new(zlist)));
        }
    } else if let Value::Object(obj) = &v {
        // Check if it's a string
        if let Some(string_obj) = obj.as_any().downcast_ref::<StringObject>() {
            // Convert string to ZList
            let s = string_obj.value();
            let mut reals = Vec::new();
            
            for ch in s.chars() {
                reals.push(ch as u32 as f64);
            }
            
            let array = Array::from_reals(reals);
            let zlist = List::from_array(array);
            thread.push(Value::Object(Arc::new(zlist)));
        } else {
            // Return other objects as-is
            thread.push(v);
        }
    } else {
        // Return other values as-is
        thread.push(v);
    }

    Ok(())
}

/// Convert anything to an infinite stream of itself (L)
/// - Lists: Return as-is
/// - Other: Create infinite stream that repeats the value
pub fn l_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(_list) = value_as_list(&v) {
        // Already a list, return as-is
        thread.push(v);
    } else {
        // Create infinite stream - for now, create a single-element list
        // TODO: Implement proper infinite stream generator when generators are available
        let values = vec![v];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));
    }

    Ok(())
}

/// Convert anything to a single-element list (L1)
/// - Lists: Return as-is
/// - Other: Wrap in a one-element list
pub fn l1_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(_list) = value_as_list(&v) {
        // Already a list, return as-is
        thread.push(v);
    } else {
        // Wrap in single-element list
        let values = vec![v];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));
    }

    Ok(())
}

/// Convert a list to a string (unspell)
/// - VList/ZList: Convert elements to characters
/// - String: Return as-is
/// - Other: Error
pub fn unspell_op(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let v = thread.pop_value()?;

    if let Some(list) = value_as_list(&v) {
        // Convert list to string
        let mut list_clone = list.clone();
        let len = list_clone.length()?;
        let mut chars = Vec::new();
        
        for i in 0..len {
            match list.at(i) {
                Ok(item) => {
                    match item.as_float() {
                        Ok(f) => {
                            let ch = f as u32;
                            if let Some(c) = char::from_u32(ch) {
                                chars.push(c);
                            } else {
                                chars.push('\0'); // Invalid character becomes null
                            }
                        }
                        Err(_) => chars.push('\0'), // Non-numeric becomes null
                    }
                }
                Err(_) => break,
            }
        }
        
        let string = chars.into_iter().collect::<String>();
        let string_obj = StringObject::new(string);
        thread.push(Value::Object(Arc::new(string_obj)));
    } else if let Value::Object(obj) = &v {
        // Check if it's already a string
        if obj.as_any().downcast_ref::<StringObject>().is_some() {
            thread.push(v);
        } else {
            return Err(SapfError::WrongType);
        }
    } else {
        return Err(SapfError::WrongType);
    }

    Ok(())
}

// ============================================================================
// Registration and Module Interface
// ============================================================================

/// Register all basic stream operations with the VM
pub fn register_stream_ops(vm: &crate::vm::VM) -> Result<(), SapfError> {
    // Basic list operations
    vm.def_by_name("size", Value::from_object(Arc::new(Primitive::new(
        size_op, Value::Real(0.0), "size",
        "(seq --> num) Return the length of a sequence if it is finite. Returns inf if indefinite.",
        1, 1
    ))))?;

    vm.def_by_name("rank", Value::from_object(Arc::new(Primitive::new(
        rank_op, Value::Real(0.0), "rank",
        "(a --> n) Return the rank of an object.",
        1, 1
    ))))?;

    vm.def_by_name("shape", Value::from_object(Arc::new(Primitive::new(
        shape_op, Value::Real(0.0), "shape",
        "(a --> [n..]) Return the shape of an object.",
        1, 1
    ))))?;

    vm.def_by_name("finite", Value::from_object(Arc::new(Primitive::new(
        finite_op, Value::Real(0.0), "finite",
        "(seq --> bool) Returns 1 if the sequence is finite, 0 if indefinite.",
        1, 1
    ))))?;

    vm.def_by_name("empty", Value::from_object(Arc::new(Primitive::new(
        empty_op, Value::Real(0.0), "empty",
        "(list --> bool) returns whether the list is empty.",
        1, 1
    ))))?;

    vm.def_by_name("nonempty", Value::from_object(Arc::new(Primitive::new(
        nonempty_op, Value::Real(0.0), "nonempty",
        "(list --> bool) returns whether the list is nonempty.",
        1, 1
    ))))?;

    vm.def_by_name("head", Value::from_object(Arc::new(Primitive::new(
        head_op, Value::Real(0.0), "head",
        "(list --> item) returns first item of list. fails if list is empty.",
        1, 1
    ))))?;

    vm.def_by_name("tail", Value::from_object(Arc::new(Primitive::new(
        tail_op, Value::Real(0.0), "tail",
        "(list --> list) returns the rest of the list after the first item.",
        1, 1
    ))))?;

    vm.def_by_name("packed", Value::from_object(Arc::new(Primitive::new(
        packed_op, Value::Real(0.0), "packed",
        "(list --> bool) returns whether the list is packed.",
        1, 1
    ))))?;

    vm.def_by_name("pack", Value::from_object(Arc::new(Primitive::new(
        pack_op, Value::Real(0.0), "pack",
        "(list --> list) returns a packed version of the list.",
        1, 1
    ))))?;

    // List conversion operations
    vm.def_by_name("V", Value::from_object(Arc::new(Primitive::new(
        v_op, Value::Real(0.0), "V",
        "(a --> VList) Convert to a Value list. ZLists become VLists, strings become character lists.",
        1, 1
    ))))?;

    vm.def_by_name("Z", Value::from_object(Arc::new(Primitive::new(
        z_op, Value::Real(0.0), "Z",
        "(a --> ZList) Convert to a Real list. VLists become ZLists, strings become ASCII code lists.",
        1, 1
    ))))?;

    vm.def_by_name("L", Value::from_object(Arc::new(Primitive::new(
        l_op, Value::Real(0.0), "L",
        "(anything --> stream) streams are returned as is. anything else is made into an infinite stream of itself.",
        1, 1
    ))))?;

    vm.def_by_name("L1", Value::from_object(Arc::new(Primitive::new(
        l1_op, Value::Real(0.0), "L1",
        "(anything --> stream) streams are returned as is. anything else is wrapped in a one item list.",
        1, 1
    ))))?;

    vm.def_by_name("unspell", Value::from_object(Arc::new(Primitive::new(
        unspell_op, Value::Real(0.0), "unspell",
        "(sequence --> string) converts a stream of numbers or a signal to a string.",
        1, 1
    ))))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{vm::VM, thread::Thread};
    use crate::core::list::{List, Array};
    use std::sync::Arc;

    fn get_test_thread() -> Thread {
        Thread::new()
    }

    #[test]
    fn test_size_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(size_op, Value::Real(0.0), "size", "test", 1, 1);

        // Test finite list
        let values = vec![Value::Real(1.0), Value::Real(2.0), Value::Real(3.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        size_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 3.0);

        // Test non-list value
        thread.push(Value::Real(42.0));
        size_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0);
    }

    #[test]
    fn test_rank_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(rank_op, Value::Real(0.0), "rank", "test", 1, 1);

        // Test scalar (rank 0)
        thread.push(Value::Real(42.0));
        rank_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 0.0);

        // Test 1D list (rank 1)
        let values = vec![Value::Real(1.0), Value::Real(2.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        rank_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0);
    }

    #[test]
    fn test_finite_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(finite_op, Value::Real(0.0), "finite", "test", 1, 1);

        // Test finite list
        let values = vec![Value::Real(1.0), Value::Real(2.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        finite_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0);

        // Test scalar (considered finite)
        thread.push(Value::Real(42.0));
        finite_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0);
    }

    #[test]
    fn test_empty_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(empty_op, Value::Real(0.0), "empty", "test", 1, 1);

        // Test empty list
        let array = Array::from_values(Vec::new());
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        empty_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0);

        // Test non-empty list
        let values = vec![Value::Real(1.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        empty_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 0.0);
    }

    #[test]
    fn test_head_tail_ops() {
        let mut thread = get_test_thread();
        let head_prim = Primitive::new(head_op, Value::Real(0.0), "head", "test", 1, 1);
        let tail_prim = Primitive::new(tail_op, Value::Real(0.0), "tail", "test", 1, 1);

        // Test with non-empty list
        let values = vec![Value::Real(10.0), Value::Real(20.0), Value::Real(30.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);

        // Test head
        thread.push(Value::Object(Arc::new(list.clone())));
        head_op(&mut thread, &head_prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 10.0);

        // Test tail - simplified to check it doesn't error
        thread.push(Value::Object(Arc::new(list)));
        tail_op(&mut thread, &tail_prim).unwrap();
        let tail_result = thread.pop_value().unwrap();
        // Basic check that we got a value back
        assert!(matches!(tail_result, Value::Object(_)));
    }

    #[test]
    fn test_pack_ops() {
        let mut thread = get_test_thread();
        let packed_prim = Primitive::new(packed_op, Value::Real(0.0), "packed", "test", 1, 1);
        let pack_prim = Primitive::new(pack_op, Value::Real(0.0), "pack", "test", 1, 1);

        // Test with list
        let values = vec![Value::Real(1.0), Value::Real(2.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);

        thread.push(Value::Object(Arc::new(list.clone())));
        packed_op(&mut thread, &packed_prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 1.0); // Lists are packed by default

        thread.push(Value::Object(Arc::new(list)));
        pack_op(&mut thread, &pack_prim).unwrap();
        let packed_result = thread.pop_value().unwrap();
        // Basic check that we got a value back
        assert!(matches!(packed_result, Value::Object(_)));
    }

    #[test]
    fn test_v_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(v_op, Value::Real(0.0), "V", "test", 1, 1);

        // Test converting a Real list to Value list
        let reals = vec![65.0, 66.0, 67.0]; // ASCII A, B, C
        let real_array = Array::from_reals(reals);
        let real_list = List::from_array(real_array);
        thread.push(Value::Object(Arc::new(real_list)));

        v_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.element_type(), ElementType::Value);
        } else {
            panic!("Expected list object");
        }

        // Test with string
        let string_obj = StringObject::new("ABC".to_string());
        thread.push(Value::Object(Arc::new(string_obj)));
        v_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.element_type(), ElementType::Value);
        } else {
            panic!("Expected list object");
        }

        // Test with scalar (should return as-is)
        thread.push(Value::Real(42.0));
        v_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 42.0);
    }

    #[test]
    fn test_z_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(z_op, Value::Real(0.0), "Z", "test", 1, 1);

        // Test converting a Value list to Real list
        let values = vec![Value::Real(65.0), Value::Real(66.0), Value::Real(67.0)];
        let value_array = Array::from_values(values);
        let value_list = List::from_array(value_array);
        thread.push(Value::Object(Arc::new(value_list)));

        z_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.element_type(), ElementType::Real);
        } else {
            panic!("Expected list object");
        }

        // Test with string
        let string_obj = StringObject::new("ABC".to_string());
        thread.push(Value::Object(Arc::new(string_obj)));
        z_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.element_type(), ElementType::Real);
        } else {
            panic!("Expected list object");
        }

        // Test with scalar (should return as-is)
        thread.push(Value::Real(42.0));
        z_op(&mut thread, &prim).unwrap();
        assert_eq!(thread.pop_real("test").unwrap(), 42.0);
    }

    #[test]
    fn test_l_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(l_op, Value::Real(0.0), "L", "test", 1, 1);

        // Test with list (should return as-is)
        let values = vec![Value::Real(1.0), Value::Real(2.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        l_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        assert!(matches!(result, Value::Object(_)));

        // Test with scalar (should wrap in list)
        thread.push(Value::Real(42.0));
        l_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.at(0).unwrap().as_float().unwrap(), 42.0);
        } else {
            panic!("Expected list object");
        }
    }

    #[test]
    fn test_l1_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(l1_op, Value::Real(0.0), "L1", "test", 1, 1);

        // Test with list (should return as-is)
        let values = vec![Value::Real(1.0), Value::Real(2.0)];
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        l1_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        assert!(matches!(result, Value::Object(_)));

        // Test with scalar (should wrap in single-element list)
        thread.push(Value::Real(42.0));
        l1_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let list = obj.as_any().downcast_ref::<List>().unwrap();
            assert_eq!(list.at(0).unwrap().as_float().unwrap(), 42.0);
        } else {
            panic!("Expected list object");
        }
    }

    #[test]
    fn test_unspell_op() {
        let mut thread = get_test_thread();
        let prim = Primitive::new(unspell_op, Value::Real(0.0), "unspell", "test", 1, 1);

        // Test converting list to string
        let values = vec![Value::Real(65.0), Value::Real(66.0), Value::Real(67.0)]; // ASCII A, B, C
        let array = Array::from_values(values);
        let list = List::from_array(array);
        thread.push(Value::Object(Arc::new(list)));

        unspell_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let string_obj = obj.as_any().downcast_ref::<StringObject>().unwrap();
            assert_eq!(string_obj.value(), "ABC");
        } else {
            panic!("Expected string object");
        }

        // Test with string (should return as-is)
        let string_obj = StringObject::new("Hello".to_string());
        thread.push(Value::Object(Arc::new(string_obj)));
        unspell_op(&mut thread, &prim).unwrap();
        let result = thread.pop_value().unwrap();
        if let Value::Object(obj) = result {
            let string_obj = obj.as_any().downcast_ref::<StringObject>().unwrap();
            assert_eq!(string_obj.value(), "Hello");
        } else {
            panic!("Expected string object");
        }
    }
}

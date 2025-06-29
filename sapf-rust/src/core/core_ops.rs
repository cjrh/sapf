//    SAPF - Sound As Pure Form
//    Copyright (C) 2019 James McCartney
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::core::{SapfError, Value, Object};
use crate::core::value::StringObject;
use crate::vm::Thread;
use std::sync::Arc;

/// Core stack operation: clear all items from stack
pub fn clear(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    thread.clear_stack();
    Ok(vec![])
}

/// Core stack operation: clear all but top item from stack  
pub fn cleard(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let top = thread.pop()?;
    thread.clear_stack();
    Ok(vec![top])
}

/// Core stack operation: get stack depth
pub fn stack_depth(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let depth = thread.stack_depth() as f64;
    Ok(vec![Value::Real(depth)])
}

/// Core stack operation: swap top two items (ba)
pub fn ba(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![b, a])
}

/// Core stack operation: swap second and third items, keep top (bac)
pub fn bac(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let c = thread.pop()?;
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![b, a, c])
}

/// Core stack operation: rotate right (cab)
pub fn cab(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let c = thread.pop()?;
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![c, a, b])
}

/// Core stack operation: rotate left (bca)
pub fn bca(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let c = thread.pop()?;
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![b, c, a])
}

/// Core stack operation: reverse top 3 (cba)
pub fn cba(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let c = thread.pop()?;
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![c, b, a])
}

/// Core stack operation: duplicate top item (aa)
pub fn aa(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let a = thread.pop()?;
    Ok(vec![a.clone(), a])
}

/// Core stack operation: duplicate top item twice (aaa)
pub fn aaa(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let a = thread.pop()?;
    Ok(vec![a.clone(), a.clone(), a])
}

/// Core stack operation: over - copy second item to top (aba)
pub fn aba(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![a.clone(), b, a])
}

/// Core stack operation: tuck - copy top item under second (bab)
pub fn bab(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![b.clone(), a, b])
}

/// Core stack operation: aab pattern
pub fn aab(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![a.clone(), a, b])
}

/// Core stack operation: aabb pattern
pub fn aabb(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![a.clone(), a, b.clone(), b])
}

/// Core stack operation: abab pattern
pub fn abab(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    Ok(vec![a.clone(), b.clone(), a, b])
}

/// Core stack operation: nip - remove second item (a b -> b)
pub fn nip(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let b = thread.pop()?;
    let _a = thread.pop()?; // discard second item
    Ok(vec![b])
}

/// Core stack operation: pop - remove top item
pub fn pop_op(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    let _top = thread.pop()?; // discard top item
    Ok(vec![])
}

/// Control flow operation: while loop
pub fn while_op(thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 2 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 2, 
            actual: args.len() 
        });
    }
    
    let body = &args[1];
    let test = &args[0];
    
    loop {
        // Save stack state and execute test
        let saved_depth = thread.stack_depth();
        
        // Apply test function
        test.apply(thread)?;
        
        // Check result
        let test_result = thread.pop()?;
        let should_continue = match test_result {
            Value::Real(x) => x == 0.0, // In SAPF, 0 is false, non-zero is true
            Value::Nil => false,
            _ => true,
        };
        
        // Restore stack to saved state
        while thread.stack_depth() > saved_depth {
            thread.pop()?;
        }
        
        if !should_continue {
            break;
        }
        
        // Save stack state and execute body
        let saved_depth = thread.stack_depth();
        
        // Apply body function
        body.apply(thread)?;
        
        // Restore stack to saved state (discard any results from body)
        while thread.stack_depth() > saved_depth {
            thread.pop()?;
        }
    }
    
    Ok(vec![])
}

/// Control flow operation: if-then-else
pub fn if_op(thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 3 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 3, 
            actual: args.len() 
        });
    }
    
    let else_code = &args[2];
    let then_code = &args[1];
    let test = &args[0];
    
    // Evaluate test condition
    let is_true = match test {
        Value::Real(x) => *x != 0.0,
        Value::Nil => false,
        _ => true,
    };
    
    // Execute appropriate branch
    if is_true {
        then_code.apply(thread)?;
    } else {
        else_code.apply(thread)?;
    }
    
    Ok(vec![])
}

/// Comparison operation: equals
pub fn equals(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 2 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 2, 
            actual: args.len() 
        });
    }
    
    let result = args[0].equals(&args[1])?;
    Ok(vec![Value::Real(if result { 1.0 } else { 0.0 })])
}

/// Comparison operation: less than
pub fn less(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 2 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 2, 
            actual: args.len() 
        });
    }
    
    use std::cmp::Ordering;
    let result = matches!(args[0].compare(&args[1])?, Ordering::Less);
    Ok(vec![Value::Real(if result { 1.0 } else { 0.0 })])
}

/// Comparison operation: greater than  
pub fn greater(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 2 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 2, 
            actual: args.len() 
        });
    }
    
    use std::cmp::Ordering;
    let result = matches!(args[0].compare(&args[1])?, Ordering::Greater);
    Ok(vec![Value::Real(if result { 1.0 } else { 0.0 })])
}

/// Logical operation: not
pub fn not_op(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 1 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 1, 
            actual: args.len() 
        });
    }
    
    let is_false = match &args[0] {
        Value::Real(x) => *x == 0.0,
        Value::Nil => true,
        _ => false,
    };
    
    Ok(vec![Value::Real(if is_false { 1.0 } else { 0.0 })])
}

/// Function application: apply
pub fn apply(thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 1 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 1, 
            actual: args.len() 
        });
    }
    
    args[0].apply(thread)?;
    Ok(vec![])
}

/// Print operation: print value
pub fn pr(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 1 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 1, 
            actual: args.len() 
        });
    }
    
    let print_str = args[0].print();
    print!("{}", print_str);
    Ok(vec![])
}

/// Print operation: carriage return
pub fn cr(_thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    println!();
    Ok(vec![])
}

/// Print operation: space
pub fn sp(_thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    print!(" ");
    Ok(vec![])
}

/// Print operation: tab
pub fn tab(_thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    print!("\t");
    Ok(vec![])
}

/// Print operation: print stack
pub fn prstk(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    println!("stack: {}", thread.print_stack());
    Ok(vec![])
}

/// Type operation: get type name
pub fn type_op(_thread: &mut Thread, args: &[Value]) -> Result<Vec<Value>, SapfError> {
    if args.len() != 1 {
        return Err(SapfError::WrongArgumentCount { 
            expected: 1, 
            actual: args.len() 
        });
    }
    
    let type_name = args[0].type_name();
    Ok(vec![Value::object(StringObject::new(type_name.to_string()))])
}

/// Audio rate operations: sample rate
pub fn sr(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    Ok(vec![Value::Real(thread.sample_rate())])
}

/// Audio rate operations: nyquist rate
pub fn nyq(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    Ok(vec![Value::Real(thread.sample_rate() * 0.5)])
}

/// Audio rate operations: inverse sample rate
pub fn isr(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    Ok(vec![Value::Real(1.0 / thread.sample_rate())])
}

/// Audio rate operations: radians per sample
pub fn rps(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    Ok(vec![Value::Real(2.0 * std::f64::consts::PI / thread.sample_rate())])
}

/// Audio rate operations: inverse nyquist rate
pub fn inyq(thread: &mut Thread, _args: &[Value]) -> Result<Vec<Value>, SapfError> {
    Ok(vec![Value::Real(2.0 / thread.sample_rate())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{Thread, VM};
    use crate::core::Value;

    fn create_test_thread() -> Thread {
        use crate::vm::thread::Rate;
        Thread::with_rate(Rate {
            sample_rate: 44100.0,
            block_size: 64,
        })
    }

    #[test]
    fn test_stack_operations() {
        let mut thread = create_test_thread();
        
        // Test basic stack operations
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));
        
        // Test stack depth
        let result = stack_depth(&mut thread, &[]).unwrap();
        assert_eq!(result[0], Value::Real(2.0));
        
        // Test ba (swap)
        let result = ba(&mut thread, &[]).unwrap();
        assert_eq!(result[0], Value::Real(2.0));
        assert_eq!(result[1], Value::Real(1.0));
        
        // Test aa (dup)  
        thread.push(Value::Real(3.0));
        let result = aa(&mut thread, &[]).unwrap();
        assert_eq!(result[0], Value::Real(3.0));
        assert_eq!(result[1], Value::Real(3.0));
    }

    #[test]
    fn test_comparison_operations() {
        let thread = create_test_thread();
        
        // Test equals
        let result = equals(&mut thread.clone(), &[Value::Real(1.0), Value::Real(1.0)]).unwrap();
        assert_eq!(result[0], Value::Real(1.0));
        
        let result = equals(&mut thread.clone(), &[Value::Real(1.0), Value::Real(2.0)]).unwrap();
        assert_eq!(result[0], Value::Real(0.0));
        
        // Test less
        let result = less(&mut thread.clone(), &[Value::Real(1.0), Value::Real(2.0)]).unwrap();
        assert_eq!(result[0], Value::Real(1.0));
        
        let result = less(&mut thread.clone(), &[Value::Real(2.0), Value::Real(1.0)]).unwrap();
        assert_eq!(result[0], Value::Real(0.0));
        
        // Test greater
        let result = greater(&mut thread.clone(), &[Value::Real(2.0), Value::Real(1.0)]).unwrap();
        assert_eq!(result[0], Value::Real(1.0));
        
        let result = greater(&mut thread.clone(), &[Value::Real(1.0), Value::Real(2.0)]).unwrap();
        assert_eq!(result[0], Value::Real(0.0));
    }

    #[test]
    fn test_logical_operations() {
        let thread = create_test_thread();
        
        // Test not
        let result = not_op(&mut thread.clone(), &[Value::Real(0.0)]).unwrap();
        assert_eq!(result[0], Value::Real(1.0));
        
        let result = not_op(&mut thread.clone(), &[Value::Real(1.0)]).unwrap();
        assert_eq!(result[0], Value::Real(0.0));
        
        let result = not_op(&mut thread.clone(), &[Value::Nil]).unwrap();
        assert_eq!(result[0], Value::Real(1.0));
    }

    #[test]
    fn test_audio_rate_operations() {
        let thread = create_test_thread();
        
        // Test sr
        let result = sr(&mut thread.clone(), &[]).unwrap();
        assert_eq!(result[0], Value::Real(44100.0));
        
        // Test nyq
        let result = nyq(&mut thread.clone(), &[]).unwrap();
        assert_eq!(result[0], Value::Real(22050.0));
        
        // Test isr
        let result = isr(&mut thread.clone(), &[]).unwrap();
        assert!((result[0].as_float().unwrap() - (1.0 / 44100.0)).abs() < 1e-10);
    }

    #[test]
    fn test_type_operation() {
        let thread = create_test_thread();
        
        let result = type_op(&mut thread.clone(), &[Value::Real(1.0)]).unwrap();
        if let Value::Object(obj) = &result[0] {
            if let Some(s) = obj.as_any().downcast_ref::<StringObject>() {
                assert_eq!(s.value(), "Real");
            } else {
                panic!("Expected StringObject");
            }
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_complex_stack_operations() {
        let mut thread = create_test_thread();
        
        // Test three-item rotations
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0)); 
        thread.push(Value::Real(3.0));
        
        // Test cab (rotate right: a b c -> c a b)
        let result = cab(&mut thread, &[]).unwrap();
        assert_eq!(result[0], Value::Real(3.0)); // c
        assert_eq!(result[1], Value::Real(1.0)); // a
        assert_eq!(result[2], Value::Real(2.0)); // b
        
        // Test bca (rotate left: a b c -> b c a)
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));
        thread.push(Value::Real(3.0));
        
        let result = bca(&mut thread, &[]).unwrap();
        assert_eq!(result[0], Value::Real(2.0)); // b
        assert_eq!(result[1], Value::Real(3.0)); // c
        assert_eq!(result[2], Value::Real(1.0)); // a
    }

    #[test]
    fn test_nip_operation() {
        let mut thread = create_test_thread();
        
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));
        
        // nip should remove second item: a b -> b
        let result = nip(&mut thread, &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Value::Real(2.0));
    }

    #[test]
    fn test_pop_operation() {
        let mut thread = create_test_thread();
        
        thread.push(Value::Real(1.0));
        
        // pop should remove top item
        let result = pop_op(&mut thread, &[]).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(thread.stack_depth(), 0);
    }

    #[test]
    fn test_clear_operations() {
        let mut thread = create_test_thread();
        
        // Set up stack with multiple items
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));
        thread.push(Value::Real(3.0));
        
        // Test clear
        let result = clear(&mut thread, &[]).unwrap();
        assert_eq!(result.len(), 0);
        assert_eq!(thread.stack_depth(), 0);
        
        // Test cleard
        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));
        thread.push(Value::Real(3.0));
        
        let result = cleard(&mut thread, &[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Value::Real(3.0));
        assert_eq!(thread.stack_depth(), 0);
    }
}
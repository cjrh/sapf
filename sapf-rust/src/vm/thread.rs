//! Thread execution context for SAPF VM
//!
//! This module implements the Thread class which provides stack-based computation
//! with integrated parsing and execution capabilities. It manages both a main
//! data stack and a local variable stack for function calls.

use std::rc::Rc;
use std::sync::Arc;
use crate::core::value::{Value, Object, StringObject};
use crate::core::error::{SapfError, Result};
use crate::core::form::GForm;
use crate::core::function::Function;
use crate::core::random_ops::RGen;

/// Stack size limit (though not actively enforced)
const STACK_SIZE: usize = 16384;

/// Audio sample rate information
#[derive(Debug, Clone)]
pub struct Rate {
    pub sample_rate: f64,
    pub block_size: usize,
}

impl Default for Rate {
    fn default() -> Self {
        Rate {
            sample_rate: 44100.0,
            block_size: 64,
        }
    }
}

/// Thread execution context for the SAPF virtual machine
/// 
/// This corresponds to the Thread class in the C++ implementation.
/// It provides stack-based computation with automatic type checking
/// and error handling.
#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub prev_stack_base: usize,
    pub prev_local_base: usize,
    pub function: Option<Arc<Function>>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct Thread {
    /// Main data stack for computation
    stack: Vec<Value>,
    /// Base index for current stack frame
    stack_base: usize,
    
    /// Local variable stack for function calls
    local: Vec<Value>,
    /// Base index for current local frame
    local_base: usize,
    
    /// Call stack for function contexts
    call_stack: Vec<FunctionContext>,
    /// Current function being executed
    current_function: Option<Arc<Function>>,
    
    /// Shared workspace for forms and variables
    workspace: Option<Arc<GForm>>,
    
    /// Audio processing context
    rate: Rate,
    
    /// Random number generator
    pub rgen: RGen,
}

impl Thread {
    /// Create a new thread with empty stacks
    pub fn new() -> Self {
        Thread {
            stack: Vec::with_capacity(STACK_SIZE),
            stack_base: 0,
            local: Vec::with_capacity(256),
            local_base: 0,
            call_stack: Vec::new(),
            current_function: None,
            workspace: None,
            rate: Rate::default(),
            rgen: RGen::new(1), // Default seed
        }
    }
    
    /// Create a thread with a specific sample rate
    pub fn with_rate(rate: Rate) -> Self {
        Thread {
            stack: Vec::with_capacity(STACK_SIZE),
            stack_base: 0,
            local: Vec::with_capacity(256),
            local_base: 0,
            call_stack: Vec::new(),
            current_function: None,
            workspace: None,
            rate,
            rgen: RGen::new(1), // Default seed
        }
    }
    
    /// Create a thread with a workspace
    pub fn with_workspace(workspace: Arc<GForm>) -> Self {
        Thread {
            stack: Vec::with_capacity(STACK_SIZE),
            stack_base: 0,
            local: Vec::with_capacity(256),
            local_base: 0,
            call_stack: Vec::new(),
            current_function: None,
            workspace: Some(workspace),
            rate: Rate::default(),
            rgen: RGen::new(1), // Default seed
        }
    }
    
    // === Basic Stack Operations ===
    
    /// Push a value onto the stack
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    
    /// Push a floating-point number onto the stack
    pub fn push_float(&mut self, f: f64) {
        self.stack.push(Value::Real(f));
    }
    
    /// Push an integer onto the stack (converted to float)
    pub fn push_int(&mut self, i: i64) {
        self.stack.push(Value::Real(i as f64));
    }
    
    /// Push a boolean onto the stack (1.0 for true, 0.0 for false)
    pub fn push_bool(&mut self, b: bool) {
        self.stack.push(Value::Real(if b { 1.0 } else { 0.0 }));
    }
    
    /// Push an object onto the stack
    pub fn push_object<T: Object + 'static>(&mut self, obj: T) {
        self.stack.push(Value::object(obj));
    }
    
    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<Value> {
        if self.stack_depth() == 0 {
            return Err(SapfError::StackUnderflow {
                expected: 1,
                actual: 0,
                operation: "pop".to_string(),
            });
        }
        Ok(self.stack.pop().unwrap())
    }
    
    /// Pop n values from the stack
    pub fn pop_n(&mut self, n: usize) -> Result<()> {
        if self.stack_depth() < n {
            return Err(SapfError::StackUnderflow {
                expected: n,
                actual: self.stack_depth(),
                operation: "pop_n".to_string(),
            });
        }
        for _ in 0..n {
            self.stack.pop();
        }
        Ok(())
    }
    
    /// Access the top of the stack without popping
    pub fn top(&self) -> Result<&Value> {
        if self.stack_depth() == 0 {
            return Err(SapfError::StackUnderflow {
                expected: 1,
                actual: 0,
                operation: "top".to_string(),
            });
        }
        Ok(self.stack.last().unwrap())
    }
    
    /// Access the top of the stack mutably without popping
    pub fn top_mut(&mut self) -> Result<&mut Value> {
        if self.stack_depth() == 0 {
            return Err(SapfError::StackUnderflow {
                expected: 1,
                actual: 0,
                operation: "top_mut".to_string(),
            });
        }
        Ok(self.stack.last_mut().unwrap())
    }
    
    // === Type-Safe Pop Operations ===
    
    /// Pop a value and dereference it
    pub fn pop_value(&mut self) -> Result<Value> {
        let val = self.pop()?;
        val.deref()
    }
    
    /// Pop a value as an integer
    pub fn pop_int(&mut self, context: &str) -> Result<i64> {
        let val = self.pop()?;
        val.as_int().map_err(|_| {
            SapfError::WrongTypeWithContext(context.to_string(), val.type_name().to_string())
        })
    }
    
    /// Pop a value as a floating-point number
    pub fn pop_float(&mut self, context: &str) -> Result<f64> {
        let val = self.pop()?;
        val.as_float().map_err(|_| {
            SapfError::WrongTypeWithContext(context.to_string(), val.type_name().to_string())
        })
    }
    
    /// Pop a value as an object
    pub fn pop_object(&mut self, context: &str) -> Result<Arc<dyn Object>> {
        let val = self.pop()?;
        val.as_object().map(|obj| Arc::clone(obj)).map_err(|_| {
            SapfError::WrongTypeWithContext(context.to_string(), val.type_name().to_string())
        })
    }
    
    /// Pop a value as a real number
    pub fn pop_real(&mut self, context: &str) -> Result<f64> {
        let val = self.pop()?;
        match val {
            Value::Real(r) => Ok(r),
            _ => Err(SapfError::WrongTypeWithContext(context.to_string(), val.type_name().to_string()))
        }
    }
    
    // === Advanced Stack Operations ===
    
    /// Insert a value n positions down from the top (tuck operation)
    pub fn tuck(&mut self, n: usize, value: Value) -> Result<()> {
        if self.stack_depth() < n {
            return Err(SapfError::StackUnderflow {
                expected: n,
                actual: self.stack_depth(),
                operation: "tuck".to_string(),
            });
        }
        
        // Add space at the top
        self.stack.push(Value::Real(0.0));
        
        // Shift elements up
        let len = self.stack.len();
        for i in 0..n {
            self.stack[len - 1 - i] = self.stack[len - 2 - i].clone();
        }
        
        // Insert the value
        self.stack[len - 1 - n] = value;
        Ok(())
    }
    
    /// Clear the entire stack
    pub fn clear_stack(&mut self) {
        self.stack.clear();
        self.stack_base = 0;
    }
    
    // === Stack Frame Management ===
    
    /// Set the stack base to an absolute position
    pub fn set_stack_base_to(&mut self, new_base: usize) {
        self.stack_base = new_base.min(self.stack.len());
    }
    
    /// Set the stack base so that 'n' elements remain visible above the base
    pub fn set_stack_base(&mut self, n: usize) {
        self.stack_base = self.stack.len().saturating_sub(n);
    }
    
    /// Get the current stack depth above the base
    pub fn stack_depth(&self) -> usize {
        self.stack.len().saturating_sub(self.stack_base)
    }
    
    /// Get the total stack size
    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }
    
    // === Local Variable Management ===
    
    /// Set the local base to an absolute position
    pub fn set_local_base_to(&mut self, new_base: usize) {
        self.local_base = new_base.min(self.local.len());
    }
    
    /// Set the local base to the current local size
    pub fn set_local_base(&mut self) {
        self.local_base = self.local.len();
    }
    
    /// Get the number of locals in the current frame
    pub fn num_locals(&self) -> usize {
        self.local.len().saturating_sub(self.local_base)
    }
    
    /// Access a local variable by index
    pub fn get_local(&self, index: usize) -> Result<&Value> {
        let actual_index = self.local_base + index;
        if actual_index >= self.local.len() {
            return Err(SapfError::OutOfRange);
        }
        Ok(&self.local[actual_index])
    }
    
    /// Access a local variable mutably by index
    pub fn get_local_mut(&mut self, index: usize) -> Result<&mut Value> {
        let actual_index = self.local_base + index;
        if actual_index >= self.local.len() {
            return Err(SapfError::OutOfRange);
        }
        Ok(&mut self.local[actual_index])
    }
    
    /// Set a local variable by index
    pub fn set_local(&mut self, index: usize, value: Value) -> Result<()> {
        let actual_index = self.local_base + index;
        if actual_index >= self.local.len() {
            // Extend the local stack if needed
            self.local.resize(actual_index + 1, Value::Real(0.0));
        }
        self.local[actual_index] = value;
        Ok(())
    }
    
    /// Push a local variable
    pub fn push_local(&mut self, value: Value) {
        self.local.push(value);
    }
    
    /// Pop locals for the current function frame
    pub fn pop_locals(&mut self) {
        self.local.truncate(self.local_base);
    }
    
    // === Workspace Management ===
    
    /// Get the current workspace
    pub fn workspace(&self) -> Option<&Arc<GForm>> {
        self.workspace.as_ref()
    }
    
    /// Set the workspace
    pub fn set_workspace(&mut self, workspace: Option<Arc<GForm>>) {
        self.workspace = workspace;
    }
    
    // === Audio Context ===
    
    /// Get the current sample rate
    pub fn sample_rate(&self) -> f64 {
        self.rate.sample_rate
    }
    
    /// Get the current block size
    pub fn block_size(&self) -> usize {
        self.rate.block_size
    }
    
    /// Set the audio rate parameters
    pub fn set_rate(&mut self, rate: Rate) {
        self.rate = rate;
    }
    
    // === Stack Inspection ===
    
    /// Get all values on the stack (for debugging)
    pub fn stack_contents(&self) -> &[Value] {
        &self.stack
    }
    
    /// Get all local variables (for debugging)
    pub fn local_contents(&self) -> &[Value] {
        &self.local
    }
    
    /// Check if stack has at least n values above base
    pub fn has_stack_depth(&self, n: usize) -> bool {
        self.stack_depth() >= n
    }
    
    /// Peek at stack value n positions from top (0 = top)
    pub fn peek(&self, n: usize) -> Result<&Value> {
        if n >= self.stack_depth() {
            return Err(SapfError::StackUnderflow {
                expected: n + 1,
                actual: self.stack_depth(),
                operation: "peek".to_string(),
            });
        }
        let index = self.stack.len() - 1 - n;
        Ok(&self.stack[index])
    }
    
    /// Peek at stack value mutably n positions from top (0 = top)
    pub fn peek_mut(&mut self, n: usize) -> Result<&mut Value> {
        if n >= self.stack_depth() {
            return Err(SapfError::StackUnderflow {
                expected: n + 1,
                actual: self.stack_depth(),
                operation: "peek_mut".to_string(),
            });
        }
        let index = self.stack.len() - 1 - n;
        Ok(&mut self.stack[index])
    }

    // === Function Context Management ===

    /// Push a function context onto the call stack
    pub fn push_function_context(&mut self, function: &Function) -> Result<()> {
        let context = FunctionContext {
            prev_stack_base: self.stack_base,
            prev_local_base: self.local_base,
            function: self.current_function.clone(),
            name: None, // Could be added later for debugging
        };

        self.call_stack.push(context);
        self.current_function = Some(Arc::new(function.clone()));

        // Set new local base for function call
        self.set_local_base();

        Ok(())
    }

    /// Pop the current function context and restore previous state
    pub fn pop_function_context(&mut self) -> Result<FunctionContext> {
        let context = self.call_stack.pop()
            .ok_or_else(|| SapfError::EmptyCallStack {
                operation: "pop_function_context".to_string(),
            })?;

        // Restore previous state
        self.stack_base = context.prev_stack_base;
        self.local_base = context.prev_local_base;
        self.current_function = context.function.clone();

        // Clean up locals from this function call
        self.local.truncate(self.local_base);

        Ok(context)
    }

    /// Get the current function being executed
    pub fn get_current_function(&self) -> Option<Arc<Function>> {
        self.current_function.clone()
    }

    /// Get the call stack depth
    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Get the stack base position
    pub fn get_stack_base(&self) -> usize {
        self.stack_base
    }

    /// Get the local base position
    pub fn get_local_base(&self) -> usize {
        self.local_base
    }

    /// Setup function parameters from stack arguments
    pub fn setup_function_params(&mut self, param_count: usize) -> Result<()> {
        if self.stack_depth() < param_count {
            return Err(SapfError::StackUnderflow {
                expected: param_count,
                actual: self.stack_depth(),
                operation: "setup_function_params".to_string(),
            });
        }

        // Move parameters from stack to locals
        let mut params = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            params.push(self.pop()?);
        }

        // Reverse the order since we popped in reverse
        params.reverse();

        // Push as locals
        for param in params {
            self.push_local(param);
        }

        Ok(())
    }

    /// Prepare return values for the caller
    pub fn prepare_return_values(&mut self, return_count: usize) -> Result<()> {
        // For now, we'll assume return values are already on the stack
        // In a more sophisticated implementation, we might need to move
        // values from locals to the stack position expected by the caller
        
        if self.stack_depth() < return_count {
            return Err(SapfError::StackUnderflow {
                expected: return_count,
                actual: self.stack_depth(),
                operation: "prepare_return_values".to_string(),
            });
        }

        Ok(())
    }
    
    /// Get a string representation of the current stack
    pub fn print_stack(&self) -> String {
        let mut result = String::new();
        result.push('[');
        
        // Print values from stack base to top
        let start = self.stack_base;
        let end = self.stack.len();
        
        for (i, idx) in (start..end).enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(&format!("{}", self.stack[idx]));
        }
        
        result.push(']');
        result
    }
    
    /// Execute bytecode on this thread
    pub fn execute_bytecode(&mut self, bytecode: &crate::vm::opcode::Bytecode) -> Result<()> {
        use crate::vm::opcode::OpCode;
        use crate::core::list::{List, Array};
        use crate::core::form::{Form, Table};
        use crate::core::symbol::get_symbol;
        
        for instruction in &bytecode.instructions {
            match instruction.opcode {
                OpCode::PushImmediate => {
                    self.push(instruction.operand.clone());
                }
                
                OpCode::PushLocalVar => {
                    // Get local variable by index
                    if let Value::Real(index) = instruction.operand {
                        let index = index as usize;
                        if index < self.local.len() {
                            self.push(self.local[index].clone());
                        } else {
                            return Err(SapfError::OutOfRange);
                        }
                    } else {
                        return Err(SapfError::WrongType);
                    }
                }
                
                OpCode::PushFunVar => {
                    // TODO: Implement function variable lookup
                    self.push(instruction.operand.clone());
                }
                
                OpCode::PushWorkspaceVar => {
                    // Lookup workspace variable by symbol
                    if let Value::Object(obj) = &instruction.operand {
                        if let Some(_string_obj) = obj.as_any().downcast_ref::<StringObject>() {
                            // Look up in workspace
                            if let Some(workspace) = &self.workspace {
                                match workspace.get(&Value::Object(obj.clone())) {
                                    Some(value) => self.push(value),
                                    None => {
                                        // Variable not found - push as symbol (late binding)
                                        self.push(instruction.operand.clone());
                                    }
                                }
                            } else {
                                // No workspace - push as symbol (late binding)
                                self.push(instruction.operand.clone());
                            }
                        } else {
                            return Err(SapfError::WrongType);
                        }
                    } else {
                        return Err(SapfError::WrongType);
                    }
                }
                
                OpCode::CallImmediate => {
                    // Call a built-in function immediately
                    if let Value::Object(obj) = &instruction.operand {
                        // Look up function in VM builtins
                        use crate::vm::vm::VM;
                        let vm = VM::instance();
                        match vm.lookup(&Value::Object(obj.clone())) {
                            Some(function_value) => {
                                // Call the function
                                if function_value.is_callable() {
                                    function_value.apply(self)?;
                                } else {
                                    // Not a function - just push the value
                                    self.push(function_value);
                                }
                            }
                            None => {
                                // Function not found - push as symbol for late binding
                                self.push(instruction.operand.clone());
                            }
                        }
                    } else {
                        return Err(SapfError::WrongType);
                    }
                }
                
                OpCode::CallLocalVar => {
                    // TODO: Implement local variable function calls
                }
                
                OpCode::CallFunVar => {
                    // TODO: Implement function variable calls
                }
                
                OpCode::CallWorkspaceVar => {
                    // TODO: Implement workspace variable calls
                }
                
                OpCode::NewVList => {
                    // Create a new value list from stack items
                    if let Value::Real(count) = instruction.operand {
                        let count = count as usize;
                        let mut values = Vec::new();
                        
                        for _ in 0..count {
                            if let Ok(value) = self.pop() {
                                values.push(value);
                            } else {
                                return Err(SapfError::StackUnderflow {
                                    expected: count,
                                    actual: 0,
                                    operation: "NewVList".to_string(),
                                });
                            }
                        }
                        values.reverse(); // Maintain order
                        
                        let array = Array::from_values(values);
                        let list = List::from_array(array);
                        self.push(Value::Object(Arc::new(list)));
                    }
                }
                
                OpCode::NewZList => {
                    // Create a new numeric list from stack items
                    if let Value::Real(count) = instruction.operand {
                        let count = count as usize;
                        let mut values = Vec::new();
                        
                        for _ in 0..count {
                            if let Ok(value) = self.pop() {
                                values.push(value);
                            } else {
                                return Err(SapfError::StackUnderflow {
                                    expected: count,
                                    actual: 0,
                                    operation: "NewZList".to_string(),
                                });
                            }
                        }
                        values.reverse(); // Maintain order
                        
                        let array = Array::from_values(values);
                        let list = List::from_array(array);
                        self.push(Value::Object(Arc::new(list)));
                    }
                }
                
                OpCode::NewForm => {
                    // Create a new form from key-value pairs on stack
                    if let Value::Real(pair_count) = instruction.operand {
                        let pair_count = pair_count as usize;
                        let mut pairs = Vec::new();
                        
                        for _ in 0..pair_count {
                            // Pop value, then key
                            let value = self.pop().map_err(|_| 
                                SapfError::StackUnderflowSimple)?;
                            let key = self.pop().map_err(|_| 
                                SapfError::StackUnderflowSimple)?;
                            pairs.push((key, value));
                        }
                        pairs.reverse(); // Maintain order
                        
                        let table = Table::from_pairs(pairs);
                        let form = Form::from_table(table);
                        self.push(Value::Object(Arc::new(form)));
                    }
                }
                
                OpCode::Return => {
                    // TODO: Implement function return
                    break; // For now, just exit the bytecode execution
                }
                
                _ => {
                    // TODO: Implement remaining opcodes
                    return Err(SapfError::CompileError(
                        format!("Unimplemented opcode: {}", instruction.opcode)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Thread {
    fn clone(&self) -> Self {
        Thread {
            stack: self.stack.clone(),
            stack_base: self.stack_base,
            local: self.local.clone(),
            local_base: self.local_base,
            call_stack: self.call_stack.clone(),
            current_function: self.current_function.clone(),
            workspace: self.workspace.clone(),
            rate: self.rate.clone(),
            rgen: self.rgen.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_stack_operations() {
        let mut thread = Thread::new();
        
        // Test push operations
        thread.push_float(42.0);
        thread.push_int(123);
        thread.push_bool(true);
        
        assert_eq!(thread.stack_depth(), 3);
        assert_eq!(thread.stack_size(), 3);
        
        // Test top access
        assert_eq!(thread.top().unwrap().as_float().unwrap(), 1.0); // bool true
        
        // Test pop operations
        let val = thread.pop().unwrap();
        assert_eq!(val.as_float().unwrap(), 1.0);
        assert_eq!(thread.stack_depth(), 2);
        
        let int_val = thread.pop_int("test").unwrap();
        assert_eq!(int_val, 123);
        
        let float_val = thread.pop_float("test").unwrap();
        assert_eq!(float_val, 42.0);
        
        assert_eq!(thread.stack_depth(), 0);
    }
    
    #[test]
    fn test_stack_underflow() {
        let mut thread = Thread::new();
        
        // Test underflow detection
        assert!(matches!(thread.pop(), Err(SapfError::StackUnderflow { .. })));
        assert!(matches!(thread.top(), Err(SapfError::StackUnderflow { .. })));
        assert!(matches!(thread.pop_n(1), Err(SapfError::StackUnderflow { .. })));
    }
    
    #[test]
    fn test_type_safe_pops() {
        let mut thread = Thread::new();
        
        // Push a string, try to pop as float
        thread.push(Value::from("not a number"));
        let result = thread.pop_float("test context");
        assert!(matches!(result, Err(SapfError::WrongTypeWithContext(_, _))));
    }
    
    #[test]
    fn test_stack_frame_management() {
        let mut thread = Thread::new();
        
        // Push some values
        thread.push_float(1.0);
        thread.push_float(2.0);
        thread.push_float(3.0);
        
        // Set stack base so 1 element remains visible 
        thread.set_stack_base(1); // Should make depth = 1
        assert_eq!(thread.stack_depth(), 1);
        
        // Set absolute base
        thread.set_stack_base_to(1);
        assert_eq!(thread.stack_depth(), 2);
        
        // Clear and check
        thread.clear_stack();
        assert_eq!(thread.stack_depth(), 0);
        assert_eq!(thread.stack_size(), 0);
    }
    
    #[test]
    fn test_tuck_operation() {
        let mut thread = Thread::new();
        
        // Push: bottom -> 1.0, 2.0, 3.0 <- top
        thread.push_float(1.0);
        thread.push_float(2.0);
        thread.push_float(3.0);
        
        // Tuck 42.0 two positions down
        thread.tuck(2, Value::Real(42.0)).unwrap();
        
        // Stack should now be: bottom -> 1.0, 42.0, 2.0, 3.0 <- top
        assert_eq!(thread.stack_depth(), 4);
        assert_eq!(thread.peek(0).unwrap().as_float().unwrap(), 3.0);  // top
        assert_eq!(thread.peek(1).unwrap().as_float().unwrap(), 2.0);
        assert_eq!(thread.peek(2).unwrap().as_float().unwrap(), 42.0); // tucked value
        assert_eq!(thread.peek(3).unwrap().as_float().unwrap(), 1.0);  // bottom
    }
    
    #[test]
    fn test_local_variables() {
        let mut thread = Thread::new();
        
        // Test local variable operations
        thread.push_local(Value::Real(10.0));
        thread.push_local(Value::Real(20.0));
        assert_eq!(thread.num_locals(), 2);
        
        // Set local base
        thread.set_local_base();
        assert_eq!(thread.num_locals(), 0);
        
        // Add more locals
        thread.push_local(Value::Real(30.0));
        assert_eq!(thread.num_locals(), 1);
        
        // Access locals
        assert_eq!(thread.get_local(0).unwrap().as_float().unwrap(), 30.0);
        
        // Set local
        thread.set_local(1, Value::Real(40.0)).unwrap();
        assert_eq!(thread.get_local(1).unwrap().as_float().unwrap(), 40.0);
        
        // Pop locals
        thread.pop_locals();
        assert_eq!(thread.num_locals(), 0);
    }
    
    #[test]
    fn test_peek_operations() {
        let mut thread = Thread::new();
        
        thread.push_float(1.0);
        thread.push_float(2.0);
        thread.push_float(3.0);
        
        assert_eq!(thread.peek(0).unwrap().as_float().unwrap(), 3.0); // top
        assert_eq!(thread.peek(1).unwrap().as_float().unwrap(), 2.0);
        assert_eq!(thread.peek(2).unwrap().as_float().unwrap(), 1.0); // bottom
        
        // Test out of bounds
        assert!(matches!(thread.peek(3), Err(SapfError::StackUnderflow { .. })));
    }
    
    #[test]
    fn test_audio_rate() {
        let rate = Rate {
            sample_rate: 48000.0,
            block_size: 128,
        };
        
        let mut thread = Thread::with_rate(rate.clone());
        assert_eq!(thread.sample_rate(), 48000.0);
        assert_eq!(thread.block_size(), 128);
        
        // Test changing rate
        let new_rate = Rate {
            sample_rate: 96000.0,
            block_size: 256,
        };
        thread.set_rate(new_rate);
        assert_eq!(thread.sample_rate(), 96000.0);
        assert_eq!(thread.block_size(), 256);
    }
    
    #[test]
    fn test_workspace() {
        let workspace = Arc::new(GForm::new());
        let mut thread = Thread::with_workspace(workspace.clone());
        
        assert!(thread.workspace().is_some());
        
        thread.set_workspace(None);
        assert!(thread.workspace().is_none());
    }
    
    #[test]
    fn test_stack_depth_check() {
        let mut thread = Thread::new();
        
        assert!(!thread.has_stack_depth(1));
        
        thread.push_float(1.0);
        assert!(thread.has_stack_depth(1));
        assert!(!thread.has_stack_depth(2));
        
        thread.push_float(2.0);
        assert!(thread.has_stack_depth(2));
    }
}
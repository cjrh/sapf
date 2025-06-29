use std::sync::Arc;
use std::fmt;
use crate::core::{Object, Value, SapfError};
use crate::core::value::StringObject;
use crate::core::form::GForm;
use crate::vm::Thread;

pub type PrimitiveFn = fn(&mut Thread, &Primitive) -> Result<(), SapfError>;

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub ops: Vec<Opcode>,
    pub constants: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct Opcode {
    pub op: u32,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub code: Arc<Bytecode>,
    pub arg_names: Vec<Arc<StringObject>>,
    pub num_args: u16,
    pub num_locals: u16,
    pub num_vars: u16,
    pub leaves: u16,
    pub workspace: Arc<GForm>,
    pub help: Option<Arc<StringObject>>,
}

impl FunctionDef {
    pub fn new(
        code: Arc<Bytecode>,
        arg_names: Vec<Arc<StringObject>>,
        num_args: u16,
        num_locals: u16,
        num_vars: u16,
        leaves: u16,
        workspace: Arc<GForm>,
        help: Option<Arc<StringObject>>,
    ) -> Self {
        Self {
            code,
            arg_names,
            num_args,
            num_locals,
            num_vars,
            leaves,
            workspace,
            help,
        }
    }

    pub fn type_name(&self) -> &'static str {
        "FunDef"
    }

    pub fn help_string(&self) -> Option<&str> {
        self.help.as_ref().map(|h| h.as_str())
    }
}

impl Object for FunctionDef {
    fn type_name(&self) -> &'static str {
        "FunDef"
    }

    fn is_function_def(&self) -> bool {
        true
    }

    fn help(&self) -> Option<&str> {
        self.help_string()
    }

    fn as_float(&self) -> Result<f64, SapfError> {
        Err(SapfError::InvalidType {
            expected: "Real".to_string(),
            found: "FunDef".to_string(),
            operation: "as_float".to_string(),
        })
    }

    fn deref(&self) -> Result<Value, SapfError> {
        Ok(Value::Object(Arc::new(self.clone())))
    }

    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub def: Arc<FunctionDef>,
    pub captured_vars: Vec<Value>,
    pub workspace: Arc<GForm>,
}

impl Function {
    pub fn new(def: Arc<FunctionDef>, captured_vars: Vec<Value>) -> Self {
        let workspace = def.workspace.clone();
        Self {
            def,
            captured_vars,
            workspace,
        }
    }

    pub fn from_stack(thread: &mut Thread, def: Arc<FunctionDef>) -> Result<Self, SapfError> {
        let num_vars = def.num_vars as usize;
        let captured_vars = if num_vars > 0 {
            if thread.stack_depth() < num_vars {
                return Err(SapfError::StackUnderflow {
                    expected: num_vars,
                    actual: thread.stack_depth(),
                    operation: "function variable capture".to_string(),
                });
            }
            let mut vars = Vec::with_capacity(num_vars);
            for _ in 0..num_vars {
                vars.push(thread.pop()?);
            }
            // Reverse since we popped in reverse order
            vars.reverse();
            vars
        } else {
            Vec::new()
        };

        Ok(Self::new(def, captured_vars))
    }

    pub fn num_args(&self) -> u16 {
        self.def.num_args
    }

    pub fn num_locals(&self) -> u16 {
        self.def.num_locals
    }

    pub fn num_vars(&self) -> u16 {
        self.def.num_vars
    }

    pub fn leaves(&self) -> u16 {
        self.def.leaves
    }

    pub fn takes(&self) -> u16 {
        self.num_args()
    }

    pub fn apply(&self, thread: &mut Thread) -> Result<(), SapfError> {
        let num_args = self.num_args() as usize;
        if thread.stack_depth() < num_args {
            return Err(SapfError::StackUnderflow {
                expected: num_args,
                actual: thread.stack_depth(),
                operation: "function application".to_string(),
            });
        }

        self.run(thread)
    }

    pub fn run(&self, thread: &mut Thread) -> Result<(), SapfError> {
        let context = FunctionContext::new(thread, self)?;
        self.execute_bytecode(thread, &context)
    }

    fn execute_bytecode(&self, thread: &mut Thread, _context: &FunctionContext) -> Result<(), SapfError> {
        for opcode in &self.def.code.ops {
            self.execute_opcode(thread, opcode)?;
        }
        Ok(())
    }

    fn execute_opcode(&self, thread: &mut Thread, opcode: &Opcode) -> Result<(), SapfError> {
        match opcode.op {
            0 => {
                thread.push(opcode.value.clone());
            }
            1 => {
                let value = thread.pop()?;
                thread.set_local(0, value)?;
            }
            2 => {
                let local = thread.get_local(0)?.clone();
                thread.push(local);
            }
            _ => {
                return Err(SapfError::InvalidOpcode {
                    opcode: opcode.op,
                    function: "execute_opcode".to_string(),
                });
            }
        }
        Ok(())
    }
}

impl Object for Function {
    fn type_name(&self) -> &'static str {
        "Fun"
    }

    fn is_function(&self) -> bool {
        true
    }

    fn is_function_or_primitive(&self) -> bool {
        true
    }

    fn help(&self) -> Option<&str> {
        self.def.help_string()
    }

    fn as_float(&self) -> Result<f64, SapfError> {
        Err(SapfError::InvalidType {
            expected: "Real".to_string(),
            found: "Fun".to_string(),
            operation: "as_float".to_string(),
        })
    }

    fn deref(&self) -> Result<Value, SapfError> {
        Ok(Value::Object(Arc::new(self.clone())))
    }

    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub function: PrimitiveFn,
    pub value: Value,
    pub name: &'static str,
    pub help: &'static str,
    pub takes: u16,
    pub leaves: u16,
}

impl Primitive {
    pub fn new(
        function: PrimitiveFn,
        value: Value,
        name: &'static str,
        help: &'static str,
        takes: u16,
        leaves: u16,
    ) -> Self {
        Self {
            function,
            value,
            name,
            help,
            takes,
            leaves,
        }
    }

    pub fn apply(&self, thread: &mut Thread) -> Result<(), SapfError> {
        let num_args = self.takes as usize;
        if thread.stack_depth() < num_args {
            return Err(SapfError::StackUnderflow {
                expected: num_args,
                actual: thread.stack_depth(),
                operation: format!("primitive function '{}'", self.name),
            });
        }

        (self.function)(thread, self)
    }

    pub fn apply_n(&self, thread: &mut Thread, n: usize) -> Result<(), SapfError> {
        if thread.stack_depth() < n {
            return Err(SapfError::StackUnderflow {
                expected: n,
                actual: thread.stack_depth(),
                operation: format!("primitive function '{}' with {} args", self.name, n),
            });
        }

        for _ in 0..n {
            self.apply(thread)?;
        }
        Ok(())
    }
}

impl Object for Primitive {
    fn type_name(&self) -> &'static str {
        "Prim"
    }

    fn is_primitive(&self) -> bool {
        true
    }

    fn is_function_or_primitive(&self) -> bool {
        true
    }

    fn help(&self) -> Option<&str> {
        Some(self.help)
    }

    fn as_float(&self) -> Result<f64, SapfError> {
        Err(SapfError::InvalidType {
            expected: "Real".to_string(),
            found: "Prim".to_string(),
            operation: "as_float".to_string(),
        })
    }

    fn deref(&self) -> Result<Value, SapfError> {
        Ok(Value::Object(Arc::new(self.clone())))
    }

    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct FunctionContext {
    previous_stack_base: usize,
    previous_local_base: usize,
    previous_function: Option<Arc<Function>>,
}

impl FunctionContext {
    fn new(thread: &mut Thread, function: &Function) -> Result<Self, SapfError> {
        let context = Self {
            previous_stack_base: thread.get_stack_base(),
            previous_local_base: thread.get_local_base(),
            previous_function: thread.get_current_function(),
        };

        thread.push_function_context(function)?;
        Ok(context)
    }
}

impl Drop for FunctionContext {
    fn drop(&mut self) {
        // Note: In a real implementation, we would need a way to restore the thread state
        // This is a simplified version that shows the pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::*;
    use crate::vm::VM;

    fn test_primitive_add(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        let b = thread.pop()?;
        let a = thread.pop()?;
        
        match (a, b) {
            (Value::Real(x), Value::Real(y)) => {
                thread.push(Value::Real(x + y));
                Ok(())
            }
            _ => Err(SapfError::InvalidType {
                expected: "Real".to_string(),
                found: "non-Real".to_string(),
                operation: "add".to_string(),
            })
        }
    }

    #[test]
    fn test_primitive_creation() {
        let prim = Primitive::new(
            test_primitive_add,
            Value::Nil,
            "add",
            "Add two numbers",
            2,
            1,
        );

        assert_eq!(prim.name, "add");
        assert_eq!(prim.help, "Add two numbers");
        assert_eq!(prim.takes, 2);
        assert_eq!(prim.leaves, 1);
        assert!(prim.is_primitive());
        assert!(prim.is_function_or_primitive());
    }

    #[test]
    fn test_primitive_application() {
        let mut thread = Thread::new();
        
        let prim = Primitive::new(
            test_primitive_add,
            Value::Nil,
            "add",
            "Add two numbers",
            2,
            1,
        );

        thread.push(Value::Real(3.0));
        thread.push(Value::Real(4.0));

        prim.apply(&mut thread).unwrap();

        let result = thread.pop().unwrap();
        assert_eq!(result, Value::Real(7.0));
    }

    #[test]
    fn test_function_def_creation() {
        let workspace = Arc::new(GForm::new());
        
        let bytecode = Arc::new(Bytecode {
            ops: vec![
                Opcode { op: 0, value: Value::Real(42.0) }, // Push constant
            ],
            constants: vec![Value::Real(42.0)],
        });

        let def = FunctionDef::new(
            bytecode,
            vec![],
            0, // num_args
            0, // num_locals
            0, // num_vars
            1, // leaves
            workspace,
            Some(Arc::new(StringObject::new("Test function".to_string()))),
        );

        assert_eq!(def.type_name(), "FunDef");
        assert_eq!(def.num_args, 0);
        assert_eq!(def.leaves, 1);
        assert!(def.is_function_def());
        assert_eq!(def.help_string(), Some("Test function"));
    }

    #[test]
    fn test_function_creation_from_def() {
        let workspace = Arc::new(GForm::new());
        
        let bytecode = Arc::new(Bytecode {
            ops: vec![
                Opcode { op: 0, value: Value::Real(42.0) },
            ],
            constants: vec![Value::Real(42.0)],
        });

        let def = Arc::new(FunctionDef::new(
            bytecode,
            vec![],
            0, 0, 0, 1,
            workspace,
            None,
        ));

        let function = Function::new(def.clone(), vec![]);

        assert_eq!(function.type_name(), "Fun");
        assert_eq!(function.num_args(), 0);
        assert_eq!(function.leaves(), 1);
        assert!(function.is_function());
        assert!(function.is_function_or_primitive());
    }

    #[test]
    fn test_stack_underflow_handling() {
        let mut thread = Thread::new();
        
        let prim = Primitive::new(
            test_primitive_add,
            Value::Nil,
            "add",
            "Add two numbers",
            2,
            1,
        );

        thread.push(Value::Real(3.0));

        let result = prim.apply(&mut thread);
        assert!(result.is_err());
        
        if let Err(SapfError::StackUnderflow { expected, actual, .. }) = result {
            assert_eq!(expected, 2);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected StackUnderflow error");
        }
    }

    #[test]
    fn test_variable_capture() {
        let mut thread = Thread::new();
        
        let workspace = Arc::new(GForm::new());
        
        let bytecode = Arc::new(Bytecode {
            ops: vec![],
            constants: vec![],
        });

        let def = Arc::new(FunctionDef::new(
            bytecode,
            vec![],
            0, 0,
            2, // num_vars - capture 2 variables
            0,
            workspace,
            None,
        ));

        thread.push(Value::Real(1.0));
        thread.push(Value::Real(2.0));

        let function = Function::from_stack(&mut thread, def).unwrap();

        assert_eq!(function.captured_vars.len(), 2);
        assert_eq!(function.captured_vars[0], Value::Real(1.0));
        assert_eq!(function.captured_vars[1], Value::Real(2.0));
        assert_eq!(thread.stack_depth(), 0);
    }
}

// Display implementations
impl fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<FunDef:{} args>", self.num_args)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Fun:{} args>", self.num_args())
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Prim:{}>", self.name)
    }
}
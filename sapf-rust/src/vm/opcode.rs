// SAPF Opcode System
//
// Bytecode opcodes for SAPF virtual machine execution

use crate::core::value::Value;
use crate::core::error::{SapfError, Result};
use std::fmt;

/// Bytecode opcodes for SAPF virtual machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    PushImmediate = 2,
    PushLocalVar = 3,
    PushFunVar = 4,
    PushWorkspaceVar = 5,
    
    // Function operations
    PushFun = 6,
    
    // Call operations
    CallImmediate = 7,
    CallLocalVar = 8,
    CallFunVar = 9,
    CallWorkspaceVar = 10,
    
    // Special operators
    Dot = 11,
    Comma = 12,
    
    // Variable binding
    BindLocal = 13,
    BindLocalFromList = 14,
    BindWorkspaceVar = 15,
    BindWorkspaceVarFromList = 16,
    
    // Data structure creation
    Parens = 17,
    NewVList = 18,
    NewZList = 19,
    NewForm = 20,
    Inherit = 21,
    Each = 22,
    
    // Control flow
    Return = 23,
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            OpCode::PushImmediate => "PushImmediate",
            OpCode::PushLocalVar => "PushLocalVar",
            OpCode::PushFunVar => "PushFunVar",
            OpCode::PushWorkspaceVar => "PushWorkspaceVar",
            OpCode::PushFun => "PushFun",
            OpCode::CallImmediate => "CallImmediate",
            OpCode::CallLocalVar => "CallLocalVar",
            OpCode::CallFunVar => "CallFunVar",
            OpCode::CallWorkspaceVar => "CallWorkspaceVar",
            OpCode::Dot => "Dot",
            OpCode::Comma => "Comma",
            OpCode::BindLocal => "BindLocal",
            OpCode::BindLocalFromList => "BindLocalFromList",
            OpCode::BindWorkspaceVar => "BindWorkspaceVar",
            OpCode::BindWorkspaceVarFromList => "BindWorkspaceVarFromList",
            OpCode::Parens => "Parens",
            OpCode::NewVList => "NewVList",
            OpCode::NewZList => "NewZList",
            OpCode::NewForm => "NewForm",
            OpCode::Inherit => "Inherit",
            OpCode::Each => "Each",
            OpCode::Return => "Return",
        };
        write!(f, "{}", name)
    }
}

/// A single bytecode instruction with operand
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: Value,
}

impl Instruction {
    pub fn new(opcode: OpCode, operand: Value) -> Self {
        Self { opcode, operand }
    }
    
    pub fn new_immediate(opcode: OpCode, value: f64) -> Self {
        Self { 
            opcode, 
            operand: Value::Real(value) 
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {:?}", self.opcode, self.operand)
    }
}

/// Compiled bytecode for execution
#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub keys: Vec<Value>,  // Keys for form construction
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            keys: Vec::new(),
        }
    }
    
    /// Add an instruction with Value operand
    pub fn add(&mut self, opcode: OpCode, operand: Value) {
        self.instructions.push(Instruction::new(opcode, operand));
    }
    
    /// Add an instruction with numeric operand
    pub fn add_immediate(&mut self, opcode: OpCode, value: f64) {
        self.instructions.push(Instruction::new_immediate(opcode, value));
    }
    
    /// Add a key for form construction
    pub fn add_key(&mut self, key: Value) {
        self.keys.push(key);
    }
    
    /// Get number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
    
    /// Append another bytecode sequence
    pub fn append(&mut self, other: &Bytecode) {
        self.instructions.extend_from_slice(&other.instructions);
        self.keys.extend_from_slice(&other.keys);
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Bytecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bytecode ({} instructions):", self.instructions.len())?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            writeln!(f, "  {}: {}", i, instruction)?;
        }
        if !self.keys.is_empty() {
            writeln!(f, "Keys ({}):", self.keys.len())?;
            for (i, key) in self.keys.iter().enumerate() {
                writeln!(f, "  {}: {:?}", i, key)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_opcode_display() {
        assert_eq!(format!("{}", OpCode::PushImmediate), "PushImmediate");
        assert_eq!(format!("{}", OpCode::CallLocalVar), "CallLocalVar");
        assert_eq!(format!("{}", OpCode::NewVList), "NewVList");
    }

    #[test]
    fn test_instruction_creation() {
        let inst = Instruction::new_immediate(OpCode::PushImmediate, 42.0);
        assert_eq!(inst.opcode, OpCode::PushImmediate);
        match inst.operand {
            Value::Real(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Real value"),
        }
    }

    #[test]
    fn test_bytecode_operations() {
        let mut bytecode = Bytecode::new();
        
        bytecode.add_immediate(OpCode::PushImmediate, 10.0);
        bytecode.add_immediate(OpCode::PushImmediate, 20.0);
        bytecode.add_immediate(OpCode::CallImmediate, 0.0);
        
        assert_eq!(bytecode.len(), 3);
        assert!(!bytecode.is_empty());
        
        let instruction = &bytecode.instructions[0];
        assert_eq!(instruction.opcode, OpCode::PushImmediate);
    }

    #[test]
    fn test_bytecode_append() {
        let mut bytecode1 = Bytecode::new();
        let mut bytecode2 = Bytecode::new();
        
        bytecode1.add_immediate(OpCode::PushImmediate, 1.0);
        bytecode2.add_immediate(OpCode::PushImmediate, 2.0);
        
        bytecode1.append(&bytecode2);
        
        assert_eq!(bytecode1.len(), 2);
        
        match &bytecode1.instructions[1].operand {
            Value::Real(n) => assert_eq!(*n, 2.0),
            _ => panic!("Expected Real value"),
        }
    }

    #[test]
    fn test_bytecode_keys() {
        let mut bytecode = Bytecode::new();
        
        bytecode.add_key(Value::Real(1.0));
        bytecode.add_key(Value::Real(2.0));
        
        assert_eq!(bytecode.keys.len(), 2);
    }
}
// SAPF Code Generation
//
// Converts AST nodes into bytecode for execution

use crate::core::error::{SapfError, Result};
use crate::core::value::{Value, Object, StringObject};
use crate::core::symbol::get_symbol;
use crate::vm::opcode::{OpCode, Bytecode};
use crate::vm::compile_scope::{CompileScope, ScopeType};
use crate::parser::parser::ASTNode;
use std::sync::Arc;

/// Code generator for converting AST to bytecode
pub struct CodeGenerator {
    // Future: Add symbol table and scope management here
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Generate bytecode from an AST node with compile scope
    pub fn generate_with_scope(&mut self, node: &ASTNode, scope: &dyn CompileScope) -> Result<Bytecode> {
        let mut bytecode = Bytecode::new();
        self.generate_node_with_scope(node, &mut bytecode, scope)?;
        Ok(bytecode)
    }
    
    /// Generate bytecode for multiple nodes with compile scope
    pub fn generate_program_with_scope(&mut self, nodes: &[ASTNode], scope: &dyn CompileScope) -> Result<Bytecode> {
        let mut bytecode = Bytecode::new();
        for node in nodes {
            self.generate_node_with_scope(node, &mut bytecode, scope)?;
        }
        Ok(bytecode)
    }
    
    /// Generate bytecode from an AST node (legacy method without scope)
    pub fn generate(&mut self, node: &ASTNode) -> Result<Bytecode> {
        let mut bytecode = Bytecode::new();
        self.generate_node(node, &mut bytecode)?;
        Ok(bytecode)
    }
    
    /// Generate bytecode for multiple nodes (legacy method without scope)
    pub fn generate_program(&mut self, nodes: &[ASTNode]) -> Result<Bytecode> {
        let mut bytecode = Bytecode::new();
        for node in nodes {
            self.generate_node(node, &mut bytecode)?;
        }
        Ok(bytecode)
    }
    
    /// Generate bytecode for a single AST node with compile scope
    fn generate_node_with_scope(&mut self, node: &ASTNode, bytecode: &mut Bytecode, scope: &dyn CompileScope) -> Result<()> {
        match node {
            ASTNode::Number(n) => {
                // Push immediate number
                bytecode.add(OpCode::PushImmediate, Value::Real(*n));
            }
            
            ASTNode::String(s) => {
                // Push immediate string object
                let string_obj = Arc::new(StringObject::from_str(s));
                bytecode.add(OpCode::PushImmediate, Value::Object(string_obj));
            }
            
            ASTNode::Symbol(sym) => {
                // Lookup variable in scope and generate appropriate opcode
                // For now, use a dummy Thread for lookup - this should be passed in properly
                use crate::vm::thread::Thread;
                use crate::vm::vm::VM;
                let vm = VM::instance();
                let thread = Thread::with_rate(vm.audio_rate().clone());
                let (scope_type, index, _value) = scope.direct_lookup(&thread, sym)?;
                match scope_type {
                    ScopeType::BuiltIn => {
                        // Built-in functions are called immediately
                        bytecode.add(OpCode::CallImmediate, Value::Object(sym.clone()));
                    }
                    ScopeType::Local => {
                        // Local variable lookup
                        bytecode.add(OpCode::PushLocalVar, Value::Real(index as f64));
                    }
                    ScopeType::FunVar => {
                        // Function closure variable lookup
                        bytecode.add(OpCode::PushFunVar, Value::Real(index as f64));
                    }
                    ScopeType::Workspace => {
                        // Workspace variable lookup
                        bytecode.add(OpCode::PushWorkspaceVar, Value::Object(sym.clone()));
                    }
                    ScopeType::Undefined => {
                        // Undefined variable - push as symbol literal (for late binding)
                        bytecode.add(OpCode::PushImmediate, Value::Object(sym.clone()));
                    }
                }
            }
            
            ASTNode::Quote(sym) => {
                // Quote prevents evaluation - push as immediate
                bytecode.add(OpCode::PushImmediate, Value::Object(sym.clone()));
            }
            
            ASTNode::List(elements) => {
                // Generate code for each element, then create list
                for elem in elements {
                    self.generate_node_with_scope(elem, bytecode, scope)?;
                }
                
                // Create VList with element count
                let count = elements.len() as f64;
                bytecode.add(OpCode::NewVList, Value::Real(count));
            }
            
            ASTNode::ZList(elements) => {
                // Generate code for each element, then create typed list
                for elem in elements {
                    self.generate_node_with_scope(elem, bytecode, scope)?;
                }
                
                // Create ZList with element count
                let count = elements.len() as f64;
                bytecode.add(OpCode::NewZList, Value::Real(count));
            }
            
            ASTNode::Form(elements) => {
                // Generate code for each key-value pair, then create form
                for elem in elements {
                    self.generate_node_with_scope(elem, bytecode, scope)?;
                }
                
                // Create Form with element count (should be even for key-value pairs)
                let count = elements.len() as f64;
                bytecode.add(OpCode::NewForm, Value::Real(count));
            }
            
            ASTNode::Lambda { args: _, help: _, body } => {
                // TODO: Implement lambda compilation with closure capture
                // For now, generate code for body
                for stmt in body {
                    self.generate_node_with_scope(stmt, bytecode, scope)?;
                }
                bytecode.add(OpCode::Return, Value::Nil);
            }
            
            ASTNode::Backquote(sym) | ASTNode::Dot(sym) | ASTNode::Comma(sym) => {
                // Special operators - push as immediate for now
                bytecode.add(OpCode::PushImmediate, Value::Object(sym.clone()));
            }
            
            ASTNode::Assignment { targets: _, is_from_list: _ } => {
                // TODO: Implement variable assignment
                return Err(SapfError::InternalError);
            }
            
            ASTNode::Call(sym) => {
                // Function call - lookup and generate call opcode
                use crate::vm::thread::Thread;
                use crate::vm::vm::VM;
                let vm = VM::instance();
                let thread = Thread::with_rate(vm.audio_rate().clone());
                let (scope_type, index, _value) = scope.direct_lookup(&thread, sym)?;
                match scope_type {
                    ScopeType::BuiltIn => {
                        bytecode.add(OpCode::CallImmediate, Value::Object(sym.clone()));
                    }
                    ScopeType::Local => {
                        bytecode.add(OpCode::CallLocalVar, Value::Real(index as f64));
                    }
                    ScopeType::FunVar => {
                        bytecode.add(OpCode::CallFunVar, Value::Real(index as f64));
                    }
                    ScopeType::Workspace => {
                        bytecode.add(OpCode::CallWorkspaceVar, Value::Object(sym.clone()));
                    }
                    ScopeType::Undefined => {
                        // Undefined function - try immediate call anyway
                        bytecode.add(OpCode::CallImmediate, Value::Object(sym.clone()));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Generate bytecode for a single AST node
    fn generate_node(&mut self, node: &ASTNode, bytecode: &mut Bytecode) -> Result<()> {
        match node {
            ASTNode::Number(n) => {
                // Push immediate number
                bytecode.add(OpCode::PushImmediate, Value::Real(*n));
            }
            
            ASTNode::String(s) => {
                // Push immediate string object
                let string_obj = Arc::new(StringObject::from_str(s));
                bytecode.add(OpCode::PushImmediate, Value::Object(string_obj));
            }
            
            ASTNode::Symbol(sym) => {
                // Push immediate symbol
                bytecode.add(OpCode::PushImmediate, Value::Object(sym.clone()));
            }
            
            ASTNode::Quote(sym) => {
                // Quote prevents evaluation - push as immediate
                bytecode.add(OpCode::PushImmediate, Value::Object(sym.clone()));
            }
            
            ASTNode::List(elements) => {
                // Generate code for each element, then create list
                for elem in elements {
                    self.generate_node(elem, bytecode)?;
                }
                
                // Create VList with element count
                let count = elements.len() as f64;
                bytecode.add(OpCode::NewVList, Value::Real(count));
            }
            
            ASTNode::ZList(elements) => {
                // Generate code for each element, then create typed list
                for elem in elements {
                    self.generate_node(elem, bytecode)?;
                }
                
                // Create ZList with element count
                let count = elements.len() as f64;
                bytecode.add(OpCode::NewZList, Value::Real(count));
            }
            
            ASTNode::Form(elements) => {
                // Generate code for key-value pairs
                if elements.len() % 2 != 0 {
                    return Err(SapfError::ParseError(
                        "Form must have even number of elements (key-value pairs)".to_string()
                    ));
                }
                
                let mut pair_count = 0;
                let mut i = 0;
                while i + 1 < elements.len() {
                    // Generate key
                    self.generate_node(&elements[i], bytecode)?;
                    // Generate value
                    self.generate_node(&elements[i + 1], bytecode)?;
                    pair_count += 1;
                    i += 2;
                }
                
                // Create form with pair count
                bytecode.add(OpCode::NewForm, Value::Real(pair_count as f64));
            }
            
            ASTNode::Lambda { args: _, help: _, body } => {
                // TODO: Implement function compilation
                // For now, generate a placeholder that creates an empty bytecode function
                for stmt in body {
                    self.generate_node(stmt, bytecode)?;
                }
                
                // Add return at end of function
                bytecode.add(OpCode::Return, Value::Real(0.0));
            }
            
            ASTNode::Call(symbol) => {
                // TODO: Implement function calls based on symbol type
                // For now, generate an immediate call
                bytecode.add(OpCode::CallImmediate, Value::Object(symbol.clone()));
            }
            
            ASTNode::Assignment { targets: _, is_from_list: _ } => {
                // TODO: Implement variable assignment opcodes
                // This requires integration with compile scope
                return Err(SapfError::CompileError(
                    "Variable assignment not yet implemented in bytecode generation".to_string()
                ));
            }
            
            ASTNode::Backquote(_) | ASTNode::Dot(_) | ASTNode::Comma(_) => {
                // TODO: Implement special operators
                return Err(SapfError::CompileError(
                    "Special operators not yet implemented in bytecode generation".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::symbol::get_symbol;

    #[test]
    fn test_generate_number() {
        let mut codegen = CodeGenerator::new();
        let node = ASTNode::Number(42.0);
        
        let bytecode = codegen.generate(&node).unwrap();
        assert_eq!(bytecode.len(), 1);
        
        let instruction = &bytecode.instructions[0];
        assert_eq!(instruction.opcode, OpCode::PushImmediate);
        match &instruction.operand {
            Value::Real(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected Real value"),
        }
    }

    #[test]
    fn test_generate_string() {
        let mut codegen = CodeGenerator::new();
        let node = ASTNode::String("hello".to_string());
        
        let bytecode = codegen.generate(&node).unwrap();
        assert_eq!(bytecode.len(), 1);
        
        let instruction = &bytecode.instructions[0];
        assert_eq!(instruction.opcode, OpCode::PushImmediate);
    }

    #[test]
    fn test_generate_list() {
        let mut codegen = CodeGenerator::new();
        let elements = vec![
            ASTNode::Number(1.0),
            ASTNode::Number(2.0),
            ASTNode::Number(3.0),
        ];
        let node = ASTNode::List(elements);
        
        let bytecode = codegen.generate(&node).unwrap();
        assert_eq!(bytecode.len(), 4); // 3 push + 1 newlist
        
        // Check the NewVList instruction
        let last_instruction = &bytecode.instructions[3];
        assert_eq!(last_instruction.opcode, OpCode::NewVList);
        match &last_instruction.operand {
            Value::Real(count) => assert_eq!(*count, 3.0),
            _ => panic!("Expected Real count"),
        }
    }

    #[test]
    fn test_generate_form() {
        let mut codegen = CodeGenerator::new();
        let key_sym = get_symbol("key");
        let elements = vec![
            ASTNode::Symbol(key_sym),
            ASTNode::Number(42.0),
        ];
        let node = ASTNode::Form(elements);
        
        let bytecode = codegen.generate(&node).unwrap();
        assert_eq!(bytecode.len(), 3); // symbol + number + newform
        
        // Check the NewForm instruction
        let last_instruction = &bytecode.instructions[2];
        assert_eq!(last_instruction.opcode, OpCode::NewForm);
        match &last_instruction.operand {
            Value::Real(count) => assert_eq!(*count, 1.0), // 1 pair
            _ => panic!("Expected Real count"),
        }
    }

    #[test]
    fn test_generate_program() {
        let mut codegen = CodeGenerator::new();
        let nodes = vec![
            ASTNode::Number(1.0),
            ASTNode::Number(2.0),
        ];
        
        let bytecode = codegen.generate_program(&nodes).unwrap();
        assert_eq!(bytecode.len(), 2);
        
        // Check both instructions are PushImmediate
        for instruction in &bytecode.instructions {
            assert_eq!(instruction.opcode, OpCode::PushImmediate);
        }
    }

    #[test]
    fn test_generate_lambda() {
        let mut codegen = CodeGenerator::new();
        let body = vec![ASTNode::Number(42.0)];
        let node = ASTNode::Lambda {
            args: vec![],
            help: None,
            body,
        };
        
        let bytecode = codegen.generate(&node).unwrap();
        assert_eq!(bytecode.len(), 2); // number + return
        
        // Check the return instruction
        let last_instruction = &bytecode.instructions[1];
        assert_eq!(last_instruction.opcode, OpCode::Return);
    }
    
    #[test]
    fn test_variable_binding_with_scope() {
        use crate::vm::compile_scope::TopCompileScope;
        use crate::vm::thread::Thread;
        use crate::core::symbol::get_symbol;
        
        let mut codegen = CodeGenerator::new();
        let mut scope = TopCompileScope::new();
        let thread = Thread::new();
        
        // Create a symbol for testing
        let test_symbol = get_symbol("test_var");
        
        // Create AST node with the symbol
        let node = ASTNode::Symbol(test_symbol.clone());
        
        // Generate bytecode with scope (this should perform variable lookup)
        let bytecode = codegen.generate_with_scope(&node, &scope).unwrap();
        assert_eq!(bytecode.len(), 1);
        
        // Since the variable is undefined in scope, it should generate PushImmediate
        let instruction = &bytecode.instructions[0];
        assert_eq!(instruction.opcode, OpCode::PushImmediate);
        
        // The operand should be the symbol object
        if let Value::Object(obj) = &instruction.operand {
            assert!(obj.as_any().downcast_ref::<crate::core::value::StringObject>().is_some());
        } else {
            panic!("Expected Object operand");
        }
    }
    
    #[test]
    fn test_function_call_with_scope() {
        use crate::vm::compile_scope::TopCompileScope;
        use crate::core::symbol::get_symbol;
        
        let mut codegen = CodeGenerator::new();
        let scope = TopCompileScope::new();
        
        // Create a symbol for function call
        let func_symbol = get_symbol("test_func");
        
        // Create AST node for function call
        let node = ASTNode::Call(func_symbol.clone());
        
        // Generate bytecode with scope
        let bytecode = codegen.generate_with_scope(&node, &scope).unwrap();
        assert_eq!(bytecode.len(), 1);
        
        // Should generate CallImmediate for undefined function
        let instruction = &bytecode.instructions[0];
        assert_eq!(instruction.opcode, OpCode::CallImmediate);
    }
}
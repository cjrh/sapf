//! Compile-time scoping system for SAPF
//!
//! This module implements the lexical scoping system used during compilation,
//! including variable binding, lookup, and scope management.

use std::rc::Rc;
use std::sync::Arc;
use crate::core::value::{Value, StringObject, Object};
use crate::core::error::{SapfError, Result};
use crate::core::symbol::{self, get_symbol};
use crate::vm::thread::Thread;
use crate::vm::vm::VM;

/// Scope types for variable resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Undefined,
    BuiltIn,
    Workspace,
    Local,
    FunVar,
}

/// Workspace variable definition
#[derive(Debug, Clone)]
pub struct WorkspaceDef {
    pub name: Arc<StringObject>,
}

/// Local variable definition
#[derive(Debug, Clone)]
pub struct LocalDef {
    pub name: Arc<StringObject>,
    pub index: usize,
    pub takes: i32,
    pub leaves: i32,
}

/// Function variable definition (closure variables)
#[derive(Debug, Clone)]
pub struct VarDef {
    pub name: Arc<StringObject>,
    pub index: usize,
    pub from_scope: ScopeType,
    pub from_index: usize,
}

/// Base trait for compile-time scopes
pub trait CompileScope: std::fmt::Debug {
    /// Get the number of local variables in this scope
    fn num_locals(&self) -> usize;
    
    /// Get the number of function variables (closure vars) in this scope
    fn num_vars(&self) -> usize;
    
    /// Check if this is a parenthesis scope (transparent for variable lookup)
    fn is_paren(&self) -> bool { false }
    
    /// Direct lookup within this scope only
    fn direct_lookup(&self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)>;
    
    /// Indirect lookup (may create closure variables)
    fn indirect_lookup(&mut self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)>;
    
    /// Bind a new variable in this scope
    fn bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)>;
    
    /// Inner bind (for nested scopes)
    fn inner_bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)>;
}

/// Top-level compilation scope (global scope)
#[derive(Debug)]
pub struct TopCompileScope {
    /// Local variables in this scope
    locals: Vec<LocalDef>,
    /// Function variables (closures) in this scope
    vars: Vec<VarDef>,
    /// Workspace variables
    workspace_vars: Vec<WorkspaceDef>,
}

impl TopCompileScope {
    /// Create a new top-level scope
    pub fn new() -> Self {
        TopCompileScope {
            locals: Vec::new(),
            vars: Vec::new(),
            workspace_vars: Vec::new(),
        }
    }
    
    /// Get workspace variables
    pub fn workspace_vars(&self) -> &[WorkspaceDef] {
        &self.workspace_vars
    }
}

impl CompileScope for TopCompileScope {
    fn num_locals(&self) -> usize {
        self.locals.len()
    }
    
    fn num_vars(&self) -> usize {
        self.vars.len()
    }
    
    fn direct_lookup(&self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        // Check locals first
        for (i, local) in self.locals.iter().enumerate() {
            if local.name.as_str() == name.as_str() {
                return Ok((ScopeType::Local, i, None));
            }
        }
        
        // Check function variables
        for (i, var) in self.vars.iter().enumerate() {
            if var.name.as_str() == name.as_str() {
                return Ok((ScopeType::FunVar, i, None));
            }
        }
        
        // Check workspace variables
        for workspace_var in &self.workspace_vars {
            if workspace_var.name.as_str() == name.as_str() {
                return Ok((ScopeType::Workspace, 0, None));
            }
        }
        
        // Check workspace
        if let Some(workspace) = th.workspace() {
            let key = Value::from_object(Arc::clone(name) as Arc<dyn Object>);
            if let Some(value) = workspace.get(&key) {
                return Ok((ScopeType::Workspace, 0, Some(value)));
            }
        }
        
        // Check built-ins
        let vm = VM::instance();
        let key = Value::from_object(Arc::clone(name) as Arc<dyn Object>);
        if let Some(value) = vm.lookup(&key) {
            return Ok((ScopeType::BuiltIn, 0, Some(value)));
        }
        
        Ok((ScopeType::Undefined, 0, None))
    }
    
    fn indirect_lookup(&mut self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        self.direct_lookup(th, name)
    }
    
    fn bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        // Check if already defined
        let (scope_type, index, _) = self.direct_lookup(th, &name)?;
        if scope_type != ScopeType::Undefined && scope_type != ScopeType::BuiltIn {
            return Ok((scope_type, index));
        }
        
        // Add to workspace variables
        let def = WorkspaceDef { name };
        self.workspace_vars.push(def);
        
        Ok((ScopeType::Workspace, 0))
    }
    
    fn inner_bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        // Check if already defined
        let (scope_type, index, _) = self.direct_lookup(th, &name)?;
        if scope_type == ScopeType::FunVar {
            return Err(SapfError::CompileError(format!(
                "Name {} is already in use in this scope as a free variable", 
                name.as_str()
            )));
        }
        
        if scope_type == ScopeType::Undefined {
            // Add new local variable
            let index = self.locals.len();
            let def = LocalDef {
                name,
                index,
                takes: 0,
                leaves: 0,
            };
            self.locals.push(def);
            return Ok((ScopeType::Local, index));
        }
        
        Ok((scope_type, index))
    }
}

/// Inner compilation scope (function scope)
#[derive(Debug)]
pub struct InnerCompileScope {
    /// Next scope in the chain
    next: Box<dyn CompileScope>,
    /// Local variables in this scope
    locals: Vec<LocalDef>,
    /// Function variables (closures) in this scope
    vars: Vec<VarDef>,
}

impl InnerCompileScope {
    /// Create a new inner scope
    pub fn new(next: Box<dyn CompileScope>) -> Self {
        InnerCompileScope {
            next,
            locals: Vec::new(),
            vars: Vec::new(),
        }
    }
}

impl CompileScope for InnerCompileScope {
    fn num_locals(&self) -> usize {
        self.locals.len()
    }
    
    fn num_vars(&self) -> usize {
        self.vars.len()
    }
    
    fn direct_lookup(&self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        // Check locals first
        for (i, local) in self.locals.iter().enumerate() {
            if local.name.as_str() == name.as_str() {
                return Ok((ScopeType::Local, i, None));
            }
        }
        
        // Check function variables
        for (i, var) in self.vars.iter().enumerate() {
            if var.name.as_str() == name.as_str() {
                return Ok((ScopeType::FunVar, i, None));
            }
        }
        
        Ok((ScopeType::Undefined, 0, None))
    }
    
    fn indirect_lookup(&mut self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        // Check this scope first
        let (scope_type, index, value) = self.direct_lookup(th, name)?;
        if scope_type != ScopeType::Undefined {
            return Ok((scope_type, index, value));
        }
        
        // Check outer scopes
        let (outer_scope, outer_index, outer_value) = self.next.indirect_lookup(th, name)?;
        if outer_scope == ScopeType::Undefined {
            return Ok((ScopeType::Undefined, 0, None));
        }
        
        // If it's a local or function variable from outer scope, create a closure variable
        if outer_scope == ScopeType::Local || outer_scope == ScopeType::FunVar {
            let index = self.vars.len();
            let def = VarDef {
                name: get_symbol(name.as_str()),
                index,
                from_scope: outer_scope,
                from_index: outer_index,
            };
            self.vars.push(def);
            return Ok((ScopeType::FunVar, index, None));
        }
        
        Ok((outer_scope, outer_index, outer_value))
    }
    
    fn bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        self.inner_bind_var(th, name)
    }
    
    fn inner_bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        // Check if already defined
        let (scope_type, index, _) = self.direct_lookup(th, &name)?;
        if scope_type == ScopeType::FunVar {
            return Err(SapfError::CompileError(format!(
                "Name {} is already in use in this scope as a free variable", 
                name.as_str()
            )));
        }
        
        if scope_type == ScopeType::Undefined {
            // Add new local variable
            let index = self.locals.len();
            let def = LocalDef {
                name,
                index,
                takes: 0,
                leaves: 0,
            };
            self.locals.push(def);
            return Ok((ScopeType::Local, index));
        }
        
        Ok((scope_type, index))
    }
}

/// Parenthesis compilation scope (transparent for variable lookup)
#[derive(Debug)]
pub struct ParenCompileScope {
    /// Next scope in the chain
    next: Box<dyn CompileScope>,
}

impl ParenCompileScope {
    /// Create a new parenthesis scope
    pub fn new(next: Box<dyn CompileScope>) -> Self {
        ParenCompileScope { next }
    }
    
    /// Get the next non-parenthesis scope
    pub fn next_non_paren(&self) -> &dyn CompileScope {
        let scope = &*self.next;
        while scope.is_paren() {
            // In a real implementation, we'd need a way to traverse the chain
            // For now, just return the immediate next scope
            break;
        }
        scope
    }
}

impl CompileScope for ParenCompileScope {
    fn num_locals(&self) -> usize {
        self.next.num_locals()
    }
    
    fn num_vars(&self) -> usize {
        self.next.num_vars()
    }
    
    fn is_paren(&self) -> bool {
        true
    }
    
    fn direct_lookup(&self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        self.next.direct_lookup(th, name)
    }
    
    fn indirect_lookup(&mut self, th: &Thread, name: &Arc<StringObject>) -> Result<(ScopeType, usize, Option<Value>)> {
        self.next.indirect_lookup(th, name)
    }
    
    fn bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        self.next.inner_bind_var(th, name)
    }
    
    fn inner_bind_var(&mut self, th: &Thread, name: Arc<StringObject>) -> Result<(ScopeType, usize)> {
        self.next.inner_bind_var(th, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::vm::VM;
    use crate::vm::thread::Thread;
    use crate::core::symbol::get_symbol;
    
    #[test]
    fn test_top_scope_creation() {
        let scope = TopCompileScope::new();
        
        assert_eq!(scope.num_locals(), 0);
        assert_eq!(scope.num_vars(), 0);
        assert!(!scope.is_paren());
        assert_eq!(scope.workspace_vars().len(), 0);
    }
    
    #[test]
    fn test_scope_variable_binding() -> Result<()> {
        let mut scope = TopCompileScope::new();
        let thread = Thread::new();
        let name = get_symbol("test_var");
        
        let (scope_type, _index) = scope.bind_var(&thread, name.clone())?;
        assert_eq!(scope_type, ScopeType::Workspace);
        assert_eq!(scope.workspace_vars().len(), 1);
        
        // Binding again should return the same result
        let (scope_type2, _index2) = scope.bind_var(&thread, name)?;
        assert_eq!(scope_type2, ScopeType::Workspace);
        assert_eq!(scope.workspace_vars().len(), 1); // Should not duplicate
        
        Ok(())
    }
    
    #[test]
    fn test_inner_scope_creation() {
        let top_scope = Box::new(TopCompileScope::new());
        let inner_scope = InnerCompileScope::new(top_scope);
        
        assert_eq!(inner_scope.num_locals(), 0);
        assert_eq!(inner_scope.num_vars(), 0);
        assert!(!inner_scope.is_paren());
    }
    
    #[test]
    fn test_paren_scope_creation() {
        let top_scope = Box::new(TopCompileScope::new());
        let paren_scope = ParenCompileScope::new(top_scope);
        
        assert_eq!(paren_scope.num_locals(), 0);
        assert_eq!(paren_scope.num_vars(), 0);
        assert!(paren_scope.is_paren());
    }
    
    #[test]
    fn test_local_variable_lookup() -> Result<()> {
        let mut scope = TopCompileScope::new();
        let thread = Thread::new();
        let name = get_symbol("local_var");
        
        // Initially undefined
        let (scope_type, _, _) = scope.direct_lookup(&thread, &name)?;
        assert_eq!(scope_type, ScopeType::Undefined);
        
        // Bind as local
        scope.inner_bind_var(&thread, name.clone())?;
        
        // Should now be found as local
        let (scope_type, index, _) = scope.direct_lookup(&thread, &name)?;
        assert_eq!(scope_type, ScopeType::Local);
        assert_eq!(index, 0);
        assert_eq!(scope.num_locals(), 1);
        
        Ok(())
    }
    
    #[test]
    fn test_builtin_lookup() -> Result<()> {
        let vm = VM::instance();
        let scope = TopCompileScope::new();
        let thread = Thread::new();
        
        // Define a built-in
        let name = "test_builtin";
        let value = Value::Real(42.0);
        vm.def_by_name(name, value.clone())?;
        
        let symbol = get_symbol(name);
        let (scope_type, _, builtin_value) = scope.direct_lookup(&thread, &symbol)?;
        
        assert_eq!(scope_type, ScopeType::BuiltIn);
        assert_eq!(builtin_value, Some(value));
        
        Ok(())
    }
}
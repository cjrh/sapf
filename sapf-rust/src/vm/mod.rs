//! Virtual Machine components for SAPF
//! 
//! This module contains the VM execution engine including stack management,
//! thread execution contexts, and bytecode interpretation.

pub mod thread;
pub mod vm;
pub mod compile_scope;
pub mod opcode;
pub mod builtins;

pub use thread::{Thread, Rate};
pub use vm::VM;
pub use compile_scope::{CompileScope, TopCompileScope, InnerCompileScope, ParenCompileScope};
pub use opcode::{OpCode, Instruction, Bytecode};
pub use builtins::{register_all_builtins, register_core_builtins, register_math_builtins};
//! Core modules for SAPF
//! 
//! This module contains the fundamental building blocks of the SAPF interpreter.

pub mod error;
pub mod form;
pub mod function;
pub mod hash;
pub mod list;
pub mod math;
pub mod math_ops;
pub mod reference;
pub mod symbol;
pub mod value;

pub use error::{SapfError, Result};
pub use form::{Form, GForm, Table, GTable, linearize_inheritance};
pub use function::{Function, FunctionDef, Primitive, Bytecode, Opcode, PrimitiveFn};
pub use list::{Array, List, ElementType};
pub use math_ops::{UnaryOp, BinaryOp, MathOps};
pub use reference::{Ref, ZRef};
pub use symbol::{get_symbol, lookup_symbol, symbol_table_stats};
pub use value::{Value, Object, IntoValue};
//! Core modules for SAPF
//! 
//! This module contains the fundamental building blocks of the SAPF interpreter.

pub mod error;
pub mod hash;
pub mod math;
pub mod value;

pub use error::{SapfError, Result};
pub use value::{Value, Object, IntoValue};
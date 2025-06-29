//! Error types and error handling for SAPF
//!
//! This module defines the error types used throughout the SAPF interpreter.

use thiserror::Error;

/// Main error type for SAPF operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SapfError {
    /// Execution was halted
    #[error("halt")]
    Halt,
    
    /// Generic failure
    #[error("failed")]
    Failed,
    
    /// Attempted an operation with indefinite result
    #[error("indefinite operation")]
    IndefiniteOperation,
    
    /// Wrong type for operation
    #[error("wrong type")]
    WrongType,
    
    /// Value out of valid range
    #[error("out of range")]
    OutOfRange,
    
    /// Syntax error in parsing
    #[error("syntax")]
    Syntax,
    
    /// Parse error with context
    #[error("parse error: {0}")]
    ParseError(String),
    
    /// Internal error (bug in interpreter)
    #[error("internal bug")]
    InternalError,
    
    /// Operation called in wrong state
    #[error("wrong state")]
    WrongState,
    
    /// Item not found
    #[error("not found")]
    NotFound,
    
    /// Stack overflow
    #[error("stack overflow")]
    StackOverflow,
    
    /// Stack underflow (simple variant)
    #[error("stack underflow")]
    StackUnderflowSimple,
    
    /// Inconsistent inheritance in forms
    #[error("inconsistent inheritance")]
    InconsistentInheritance,
    
    /// Undefined operation
    #[error("undefined operation")]
    UndefinedOperation,
    
    /// User requested quit
    #[error("user quit")]
    UserQuit,
    
    /// Wrong type with context information
    #[error("Wrong type in {0}: expected {1}")]
    WrongTypeWithContext(String, String),
    
    /// Compilation error during parsing or code generation
    #[error("Compilation error: {0}")]
    CompileError(String),
    
    /// Invalid value provided
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    /// Stack underflow with detailed context
    #[error("Stack underflow in {operation}: expected {expected}, got {actual}")]
    StackUnderflow {
        expected: usize,
        actual: usize,
        operation: String,
    },
    
    /// Invalid type with detailed context
    #[error("Invalid type in {operation}: expected {expected}, found {found}")]
    InvalidType {
        expected: String,
        found: String,
        operation: String,
    },
    
    /// Invalid opcode in function execution
    #[error("Invalid opcode {opcode} in {function}")]
    InvalidOpcode {
        opcode: u32,
        function: String,
    },
    
    /// Empty call stack error
    #[error("Empty call stack in {operation}")]
    EmptyCallStack {
        operation: String,
    },
    
    /// Wrong number of arguments provided
    #[error("Wrong number of arguments: expected {expected}, got {actual}")]
    WrongArgumentCount {
        expected: usize,
        actual: usize,
    },
}

/// Result type alias for SAPF operations
pub type Result<T> = std::result::Result<T, SapfError>;

impl SapfError {
    /// Get the error code (for C++ compatibility if needed)
    pub fn error_code(&self) -> i32 {
        match self {
            SapfError::Halt => -1000,
            SapfError::Failed => -1001,
            SapfError::IndefiniteOperation => -1002,
            SapfError::WrongType => -1003,
            SapfError::OutOfRange => -1004,
            SapfError::Syntax => -1005,
            SapfError::InternalError => -1006,
            SapfError::WrongState => -1007,
            SapfError::NotFound => -1008,
            SapfError::StackOverflow => -1009,
            SapfError::StackUnderflowSimple => -1010,
            SapfError::InconsistentInheritance => -1011,
            SapfError::UndefinedOperation => -1012,
            SapfError::UserQuit => -1013,
            SapfError::WrongTypeWithContext(_, _) => -1014,
            SapfError::CompileError(_) => -1015,
            SapfError::InvalidValue(_) => -1016,
            SapfError::StackUnderflow { .. } => -1017,
            SapfError::InvalidType { .. } => -1018,
            SapfError::InvalidOpcode { .. } => -1019,
            SapfError::EmptyCallStack { .. } => -1020,
            SapfError::WrongArgumentCount { .. } => -1021,
            SapfError::ParseError(_) => -1022,
        }
    }
    
    /// Create error from error code (for C++ compatibility if needed)
    pub fn from_error_code(code: i32) -> Option<Self> {
        match code {
            -1000 => Some(SapfError::Halt),
            -1001 => Some(SapfError::Failed),
            -1002 => Some(SapfError::IndefiniteOperation),
            -1003 => Some(SapfError::WrongType),
            -1004 => Some(SapfError::OutOfRange),
            -1005 => Some(SapfError::Syntax),
            -1006 => Some(SapfError::InternalError),
            -1007 => Some(SapfError::WrongState),
            -1008 => Some(SapfError::NotFound),
            -1009 => Some(SapfError::StackOverflow),
            -1010 => Some(SapfError::StackUnderflowSimple),
            -1011 => Some(SapfError::InconsistentInheritance),
            -1012 => Some(SapfError::UndefinedOperation),
            -1013 => Some(SapfError::UserQuit),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        assert_eq!(SapfError::Halt.to_string(), "halt");
        assert_eq!(SapfError::WrongType.to_string(), "wrong type");
        assert_eq!(SapfError::StackOverflow.to_string(), "stack overflow");
    }
    
    #[test]
    fn test_error_codes() {
        assert_eq!(SapfError::Halt.error_code(), -1000);
        assert_eq!(SapfError::UserQuit.error_code(), -1013);
    }
    
    #[test]
    fn test_from_error_code() {
        assert_eq!(SapfError::from_error_code(-1000), Some(SapfError::Halt));
        assert_eq!(SapfError::from_error_code(-1013), Some(SapfError::UserQuit));
        assert_eq!(SapfError::from_error_code(0), None);
        assert_eq!(SapfError::from_error_code(-999), None);
    }
    
    #[test]
    fn test_error_roundtrip() {
        for code in -1013..=-1000 {
            if let Some(error) = SapfError::from_error_code(code) {
                assert_eq!(error.error_code(), code);
            }
        }
    }
}
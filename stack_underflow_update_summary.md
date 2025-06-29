# StackUnderflow Update Summary

## Overview
Updated all instances of the old simple `SapfError::StackUnderflow` variant to use the new detailed variant `SapfError::StackUnderflow { expected, actual, operation }` in the SAPF Rust codebase.

## Files Updated

### 1. `/home/caleb/Documents/repos/sapf/sapf-rust/src/vm/thread.rs`
Updated **7 instances** of the old simple variant:

1. **Line 143** - `pop()` method: 
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: 1, actual: 0, operation: "pop".to_string() })`

2. **Line 155** - `pop_n()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: n, actual: self.stack_depth(), operation: "pop_n".to_string() })`

3. **Line 170** - `top()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: 1, actual: 0, operation: "top".to_string() })`

4. **Line 182** - `top_mut()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: 1, actual: 0, operation: "top_mut".to_string() })`

5. **Line 228** - `tuck()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: n, actual: self.stack_depth(), operation: "tuck".to_string() })`

6. **Line 382** - `peek()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: n + 1, actual: self.stack_depth(), operation: "peek".to_string() })`

7. **Line 395** - `peek_mut()` method:
   - Before: `Err(SapfError::StackUnderflow)`
   - After: `Err(SapfError::StackUnderflow { expected: n + 1, actual: self.stack_depth(), operation: "peek_mut".to_string() })`

### 2. `/home/caleb/Documents/repos/sapf/sapf-rust/src/core/error.rs`
Updated **1 instance** on line 152:
- Before: `-1010 => Some(SapfError::StackUnderflow)`
- After: `-1010 => Some(SapfError::StackUnderflowSimple)`

This was in the `from_error_code` function and needed to reference the simple variant for compatibility.

### 3. Test Updates
Updated test pattern matching in `thread.rs` to work with the new variant structure:
- Changed `Err(SapfError::StackUnderflow)` to `Err(SapfError::StackUnderflow { .. })`

### 4. Additional Fixes
Fixed missing fields in the `Clone` implementation for `Thread` struct:
- Added `call_stack` and `current_function` fields to the clone method

## Files Already Correct
### `/home/caleb/Documents/repos/sapf/sapf-rust/src/core/function.rs`
This file was already properly using the detailed `StackUnderflow` variant with appropriate context information:
- Function variable capture (line 100)
- Function application (line 138)
- Primitive function calls (lines 234, 246)

## Key Improvements
1. **Better Error Context**: Each stack underflow error now includes:
   - `expected`: Number of stack elements required
   - `actual`: Actual number of stack elements available
   - `operation`: Name of the operation that failed

2. **Improved Debugging**: Developers can now see exactly which operation failed and by how much the stack was insufficient.

3. **Consistent Error Handling**: All stack operations now provide detailed error information in a uniform format.

## Example Error Messages
The new detailed errors provide much more informative messages:
- Old: "stack underflow"
- New: "Stack underflow in pop: expected 1, got 0"
- New: "Stack underflow in primitive function 'add': expected 2, got 1"

All changes maintain backward compatibility while significantly improving error reporting and debugging capabilities.
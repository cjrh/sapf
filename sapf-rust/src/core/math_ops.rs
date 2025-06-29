use std::sync::Arc;
use crate::core::{Value, SapfError};
use crate::vm::Thread;

/// Trait for unary mathematical operations
pub trait UnaryOp: Send + Sync {
    /// Perform the scalar operation on a single float value
    fn op(&self, a: f64) -> f64;
    
    /// Get the name of this operation (for debugging/help)
    fn name(&self) -> &'static str;
    
    /// Apply the operation to a single Value (used by implementations)
    fn apply_impl(&self, thread: &mut Thread, a: Value) -> Result<Value, SapfError> where Self: Sized {
        match a {
            Value::Real(f) => Ok(Value::Real(self.op(f))),
            Value::Object(obj) => {
                // Check if it's a list or array that needs element-wise operation
                obj.unary_op(thread, self)
            }
            Value::Nil => Ok(Value::Nil), // Nil propagates through most operations
        }
    }
    
    /// Batch operation on arrays of floats (for performance)
    fn loop_floats(&self, input: &[f64], output: &mut [f64]) {
        for (i, &a) in input.iter().enumerate() {
            if i >= output.len() { break; }
            output[i] = self.op(a);
        }
    }
    
    /// Check if this operation should propagate nil values
    fn propagates_nil(&self) -> bool {
        true // Most operations propagate nil
    }
}

/// Trait for binary mathematical operations
pub trait BinaryOp: Send + Sync {
    /// Perform the scalar operation on two float values
    fn op(&self, a: f64, b: f64) -> f64;
    
    /// Get the name of this operation (for debugging/help)
    fn name(&self) -> &'static str;
    
    /// Apply the operation to two Values (used by implementations)
    fn apply_impl(&self, thread: &mut Thread, a: Value, b: Value) -> Result<Value, SapfError> where Self: Sized {
        match (a, b) {
            (Value::Real(x), Value::Real(y)) => Ok(Value::Real(self.op(x, y))),
            (Value::Real(x), Value::Object(obj)) => {
                // Real op Object - delegate to object for proper dispatch
                obj.binary_op_with_real(thread, self, x)
            }
            (Value::Object(obj), b) => {
                // Object op Value - delegate to object
                obj.binary_op(thread, self, b)
            }
            (Value::Nil, _) | (_, Value::Nil) => {
                if self.propagates_nil() {
                    Ok(Value::Nil)
                } else {
                    self.apply_impl(thread, Value::Real(0.0), Value::Real(0.0))
                }
            }
        }
    }
    
    /// Batch operation on arrays of floats (for performance)
    fn loop_floats(&self, a: &[f64], b: &[f64], output: &mut [f64]) {
        let len = a.len().min(b.len()).min(output.len());
        for i in 0..len {
            output[i] = self.op(a[i], b[i]);
        }
    }
    
    /// Batch operation with scalar and array
    fn loop_scalar_array(&self, scalar: f64, array: &[f64], output: &mut [f64]) {
        let len = array.len().min(output.len());
        for i in 0..len {
            output[i] = self.op(scalar, array[i]);
        }
    }
    
    /// Batch operation with array and scalar
    fn loop_array_scalar(&self, array: &[f64], scalar: f64, output: &mut [f64]) {
        let len = array.len().min(output.len());
        for i in 0..len {
            output[i] = self.op(array[i], scalar);
        }
    }
    
    /// Reducing operation - collapse array to single value
    fn reduce(&self, thread: &mut Thread, init: Value, array: &[Value]) -> Result<Value, SapfError> where Self: Sized {
        let mut result = init;
        for value in array {
            result = self.apply_impl(thread, result, value.clone())?;
        }
        Ok(result)
    }
    
    /// Scanning operation - progressive accumulation
    fn scan(&self, thread: &mut Thread, init: Value, array: &[Value], output: &mut Vec<Value>) -> Result<(), SapfError> where Self: Sized {
        let mut acc = init;
        output.clear();
        output.reserve(array.len());
        
        for value in array {
            acc = self.apply_impl(thread, acc.clone(), value.clone())?;
            output.push(acc.clone());
        }
        Ok(())
    }
    
    /// Pairwise operation - apply to adjacent elements
    fn pairs(&self, thread: &mut Thread, array: &[Value], output: &mut Vec<Value>) -> Result<(), SapfError> where Self: Sized {
        output.clear();
        if array.len() < 2 {
            return Ok(());
        }
        
        output.reserve(array.len() - 1);
        for i in 0..array.len() - 1 {
            let result = self.apply_impl(thread, array[i].clone(), array[i + 1].clone())?;
            output.push(result);
        }
        Ok(())
    }
    
    /// Check if this operation should propagate nil values
    fn propagates_nil(&self) -> bool {
        true // Most operations propagate nil
    }
    
    /// Check if this operation is commutative (a op b == b op a)
    fn is_commutative(&self) -> bool {
        false // Most operations are not commutative
    }
    
    /// Check if this operation is associative ((a op b) op c == a op (b op c))
    fn is_associative(&self) -> bool {
        false // Most operations are not associative
    }
    
    /// Get the identity element for this operation (if any)
    fn identity(&self) -> Option<f64> {
        None // Most operations don't have an identity
    }
    
    /// Check if this is an identity operation with the given value
    fn is_identity_with(&self, value: f64) -> bool {
        if let Some(id) = self.identity() {
            (value - id).abs() < f64::EPSILON
        } else {
            false
        }
    }
}

// Basic arithmetic operations

#[derive(Debug, Clone)]
pub struct AddOp;

impl UnaryOp for AddOp {
    fn op(&self, a: f64) -> f64 { a } // Unary + is identity
    fn name(&self) -> &'static str { "+" }
}

impl BinaryOp for AddOp {
    fn op(&self, a: f64, b: f64) -> f64 { a + b }
    fn name(&self) -> &'static str { "+" }
    fn is_commutative(&self) -> bool { true }
    fn is_associative(&self) -> bool { true }
    fn identity(&self) -> Option<f64> { Some(0.0) }
}

#[derive(Debug, Clone)]
pub struct SubOp;

impl UnaryOp for SubOp {
    fn op(&self, a: f64) -> f64 { -a } // Unary - is negation
    fn name(&self) -> &'static str { "-" }
}

impl BinaryOp for SubOp {
    fn op(&self, a: f64, b: f64) -> f64 { a - b }
    fn name(&self) -> &'static str { "-" }
}

#[derive(Debug, Clone)]
pub struct MulOp;

impl BinaryOp for MulOp {
    fn op(&self, a: f64, b: f64) -> f64 { a * b }
    fn name(&self) -> &'static str { "*" }
    fn is_commutative(&self) -> bool { true }
    fn is_associative(&self) -> bool { true }
    fn identity(&self) -> Option<f64> { Some(1.0) }
}

#[derive(Debug, Clone)]
pub struct DivOp;

impl BinaryOp for DivOp {
    fn op(&self, a: f64, b: f64) -> f64 { 
        if b == 0.0 {
            if a > 0.0 { f64::INFINITY }
            else if a < 0.0 { f64::NEG_INFINITY }
            else { f64::NAN }
        } else {
            a / b
        }
    }
    fn name(&self) -> &'static str { "/" }
}

#[derive(Debug, Clone)]
pub struct ModOp;

impl BinaryOp for ModOp {
    fn op(&self, a: f64, b: f64) -> f64 {
        if b == 0.0 {
            f64::NAN
        } else {
            a % b
        }
    }
    fn name(&self) -> &'static str { "%" }
}

#[derive(Debug, Clone)]
pub struct PowOp;

impl BinaryOp for PowOp {
    fn op(&self, a: f64, b: f64) -> f64 { a.powf(b) }
    fn name(&self) -> &'static str { "^" }
}

// Mathematical functions

#[derive(Debug, Clone)]
pub struct AbsOp;

impl UnaryOp for AbsOp {
    fn op(&self, a: f64) -> f64 { a.abs() }
    fn name(&self) -> &'static str { "abs" }
}

#[derive(Debug, Clone)]
pub struct SqrtOp;

impl UnaryOp for SqrtOp {
    fn op(&self, a: f64) -> f64 { a.sqrt() }
    fn name(&self) -> &'static str { "sqrt" }
}

#[derive(Debug, Clone)]
pub struct SinOp;

impl UnaryOp for SinOp {
    fn op(&self, a: f64) -> f64 { a.sin() }
    fn name(&self) -> &'static str { "sin" }
}

#[derive(Debug, Clone)]
pub struct CosOp;

impl UnaryOp for CosOp {
    fn op(&self, a: f64) -> f64 { a.cos() }
    fn name(&self) -> &'static str { "cos" }
}

#[derive(Debug, Clone)]
pub struct TanOp;

impl UnaryOp for TanOp {
    fn op(&self, a: f64) -> f64 { a.tan() }
    fn name(&self) -> &'static str { "tan" }
}

#[derive(Debug, Clone)]
pub struct ExpOp;

impl UnaryOp for ExpOp {
    fn op(&self, a: f64) -> f64 { a.exp() }
    fn name(&self) -> &'static str { "exp" }
}

#[derive(Debug, Clone)]
pub struct LogOp;

impl UnaryOp for LogOp {
    fn op(&self, a: f64) -> f64 { a.ln() }
    fn name(&self) -> &'static str { "log" }
}

// Comparison operations

#[derive(Debug, Clone)]
pub struct LtOp;

impl BinaryOp for LtOp {
    fn op(&self, a: f64, b: f64) -> f64 { if a < b { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { "<" }
}

#[derive(Debug, Clone)]
pub struct LeOp;

impl BinaryOp for LeOp {
    fn op(&self, a: f64, b: f64) -> f64 { if a <= b { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { "<=" }
}

#[derive(Debug, Clone)]
pub struct GtOp;

impl BinaryOp for GtOp {
    fn op(&self, a: f64, b: f64) -> f64 { if a > b { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { ">" }
}

#[derive(Debug, Clone)]
pub struct GeOp;

impl BinaryOp for GeOp {
    fn op(&self, a: f64, b: f64) -> f64 { if a >= b { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { ">=" }
}

#[derive(Debug, Clone)]
pub struct EqOp;

impl BinaryOp for EqOp {
    fn op(&self, a: f64, b: f64) -> f64 { if (a - b).abs() < f64::EPSILON { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { "==" }
    fn is_commutative(&self) -> bool { true }
}

#[derive(Debug, Clone)]
pub struct NeOp;

impl BinaryOp for NeOp {
    fn op(&self, a: f64, b: f64) -> f64 { if (a - b).abs() >= f64::EPSILON { 1.0 } else { 0.0 } }
    fn name(&self) -> &'static str { "!=" }
    fn is_commutative(&self) -> bool { true }
}

// Min/Max operations

#[derive(Debug, Clone)]
pub struct MinOp;

impl BinaryOp for MinOp {
    fn op(&self, a: f64, b: f64) -> f64 { a.min(b) }
    fn name(&self) -> &'static str { "min" }
    fn is_commutative(&self) -> bool { true }
    fn is_associative(&self) -> bool { true }
}

#[derive(Debug, Clone)]
pub struct MaxOp;

impl BinaryOp for MaxOp {
    fn op(&self, a: f64, b: f64) -> f64 { a.max(b) }
    fn name(&self) -> &'static str { "max" }
    fn is_commutative(&self) -> bool { true }
    fn is_associative(&self) -> bool { true }
}

// Additional math functions

#[derive(Debug, Clone)]
pub struct Log2Op;

impl UnaryOp for Log2Op {
    fn op(&self, a: f64) -> f64 { a.abs().log2() }
    fn name(&self) -> &'static str { "log2" }
}

#[derive(Debug, Clone)]
pub struct Log10Op;

impl UnaryOp for Log10Op {
    fn op(&self, a: f64) -> f64 { a.abs().log10() }
    fn name(&self) -> &'static str { "log10" }
}

#[derive(Debug, Clone)]
pub struct Exp2Op;

impl UnaryOp for Exp2Op {
    fn op(&self, a: f64) -> f64 { a.exp2() }
    fn name(&self) -> &'static str { "exp2" }
}

#[derive(Debug, Clone)]
pub struct FloorOp;

impl UnaryOp for FloorOp {
    fn op(&self, a: f64) -> f64 { a.floor() }
    fn name(&self) -> &'static str { "floor" }
}

#[derive(Debug, Clone)]
pub struct CeilOp;

impl UnaryOp for CeilOp {
    fn op(&self, a: f64) -> f64 { a.ceil() }
    fn name(&self) -> &'static str { "ceil" }
}

#[derive(Debug, Clone)]
pub struct RoundOp;

impl UnaryOp for RoundOp {
    fn op(&self, a: f64) -> f64 { a.round() }
    fn name(&self) -> &'static str { "round" }
}

#[derive(Debug, Clone)]
pub struct TruncOp;

impl UnaryOp for TruncOp {
    fn op(&self, a: f64) -> f64 { a.trunc() }
    fn name(&self) -> &'static str { "trunc" }
}

#[derive(Debug, Clone)]
pub struct FractOp;

impl UnaryOp for FractOp {
    fn op(&self, a: f64) -> f64 { a.fract() }
    fn name(&self) -> &'static str { "fract" }
}

#[derive(Debug, Clone)]
pub struct SignOp;

impl UnaryOp for SignOp {
    fn op(&self, a: f64) -> f64 { 
        if a < 0.0 { -1.0 }
        else if a > 0.0 { 1.0 }
        else { 0.0 }
    }
    fn name(&self) -> &'static str { "sign" }
}

// DSP and Audio-specific operations

#[derive(Debug, Clone)]
pub struct AmpDbOp;

impl UnaryOp for AmpDbOp {
    fn op(&self, a: f64) -> f64 { 
        20.0 * a.abs().log10()
    }
    fn name(&self) -> &'static str { "ampdb" }
}

#[derive(Debug, Clone)]
pub struct DbAmpOp;

impl UnaryOp for DbAmpOp {
    fn op(&self, a: f64) -> f64 { 
        10.0_f64.powf(a * 0.05)
    }
    fn name(&self) -> &'static str { "dbamp" }
}

#[derive(Debug, Clone)]
pub struct MidiCpsOp;

impl UnaryOp for MidiCpsOp {
    fn op(&self, a: f64) -> f64 { 
        440.0 * 2.0_f64.powf((a - 69.0) / 12.0)
    }
    fn name(&self) -> &'static str { "midicps" }
}

#[derive(Debug, Clone)]
pub struct CpsMidiOp;

impl UnaryOp for CpsMidiOp {
    fn op(&self, a: f64) -> f64 { 
        69.0 + 12.0 * (a / 440.0).log2()
    }
    fn name(&self) -> &'static str { "cpsmidi" }
}

#[derive(Debug, Clone)]
pub struct DistortOp;

impl UnaryOp for DistortOp {
    fn op(&self, a: f64) -> f64 { 
        a / (1.0 + a.abs())
    }
    fn name(&self) -> &'static str { "distort" }
}

#[derive(Debug, Clone)]
pub struct SoftClipOp;

impl UnaryOp for SoftClipOp {
    fn op(&self, a: f64) -> f64 { 
        let absx = a.abs();
        if absx <= 0.5 { 
            a 
        } else { 
            (absx - 0.25) / a
        }
    }
    fn name(&self) -> &'static str { "softclip" }
}

#[derive(Debug, Clone)]
pub struct TanhOp;

impl UnaryOp for TanhOp {
    fn op(&self, a: f64) -> f64 { a.tanh() }
    fn name(&self) -> &'static str { "tanh" }
}

#[derive(Debug, Clone)]
pub struct SinhOp;

impl UnaryOp for SinhOp {
    fn op(&self, a: f64) -> f64 { a.sinh() }
    fn name(&self) -> &'static str { "sinh" }
}

#[derive(Debug, Clone)]
pub struct CoshOp;

impl UnaryOp for CoshOp {
    fn op(&self, a: f64) -> f64 { a.cosh() }
    fn name(&self) -> &'static str { "cosh" }
}

// Range operations

#[derive(Debug, Clone)]
pub struct ClipOp;

impl BinaryOp for ClipOp {
    fn op(&self, a: f64, b: f64) -> f64 { 
        a.clamp(-b, b)
    }
    fn name(&self) -> &'static str { "clip2" }
}

#[derive(Debug, Clone)]
pub struct WrapOp;

impl BinaryOp for WrapOp {
    fn op(&self, a: f64, b: f64) -> f64 { 
        let range = 2.0 * b;
        if range == 0.0 { return -b; }
        let wrapped = a - range * ((a + b) / range).floor();
        wrapped
    }
    fn name(&self) -> &'static str { "wrap2" }
}

#[derive(Debug, Clone)]
pub struct FoldOp;

impl BinaryOp for FoldOp {
    fn op(&self, a: f64, b: f64) -> f64 { 
        let lo = -b;
        let hi = b;
        if hi == lo { return lo; }
        
        let x = a - lo;
        let range = hi - lo;
        let range2 = range + range;
        let c = x - range2 * (x / range2).floor();
        
        if c >= range { 
            range2 - c + lo
        } else { 
            c + lo
        }
    }
    fn name(&self) -> &'static str { "fold2" }
}

// Operation registry for lookups
pub struct MathOps {
    // We'll use static instances for efficiency
}

impl MathOps {
    /// Get a unary operation by name
    pub fn get_unary_op(name: &str) -> Option<Box<dyn UnaryOp>> {
        match name {
            "+" => Some(Box::new(AddOp)),
            "-" => Some(Box::new(SubOp)),
            "abs" => Some(Box::new(AbsOp)),
            "sqrt" => Some(Box::new(SqrtOp)),
            "sin" => Some(Box::new(SinOp)),
            "cos" => Some(Box::new(CosOp)),
            "tan" => Some(Box::new(TanOp)),
            "exp" => Some(Box::new(ExpOp)),
            "log" => Some(Box::new(LogOp)),
            "log2" => Some(Box::new(Log2Op)),
            "log10" => Some(Box::new(Log10Op)),
            "exp2" => Some(Box::new(Exp2Op)),
            "floor" => Some(Box::new(FloorOp)),
            "ceil" => Some(Box::new(CeilOp)),
            "round" => Some(Box::new(RoundOp)),
            "trunc" => Some(Box::new(TruncOp)),
            "fract" => Some(Box::new(FractOp)),
            "sign" => Some(Box::new(SignOp)),
            "ampdb" => Some(Box::new(AmpDbOp)),
            "dbamp" => Some(Box::new(DbAmpOp)),
            "midicps" => Some(Box::new(MidiCpsOp)),
            "cpsmidi" => Some(Box::new(CpsMidiOp)),
            "distort" => Some(Box::new(DistortOp)),
            "softclip" => Some(Box::new(SoftClipOp)),
            "tanh" => Some(Box::new(TanhOp)),
            "sinh" => Some(Box::new(SinhOp)),
            "cosh" => Some(Box::new(CoshOp)),
            _ => None,
        }
    }
    
    /// Get a binary operation by name
    pub fn get_binary_op(name: &str) -> Option<Box<dyn BinaryOp>> {
        match name {
            "+" => Some(Box::new(AddOp)),
            "-" => Some(Box::new(SubOp)),
            "*" => Some(Box::new(MulOp)),
            "/" => Some(Box::new(DivOp)),
            "%" => Some(Box::new(ModOp)),
            "^" => Some(Box::new(PowOp)),
            "<" => Some(Box::new(LtOp)),
            "<=" => Some(Box::new(LeOp)),
            ">" => Some(Box::new(GtOp)),
            ">=" => Some(Box::new(GeOp)),
            "==" => Some(Box::new(EqOp)),
            "!=" => Some(Box::new(NeOp)),
            "min" => Some(Box::new(MinOp)),
            "max" => Some(Box::new(MaxOp)),
            "clip2" => Some(Box::new(ClipOp)),
            "wrap2" => Some(Box::new(WrapOp)),
            "fold2" => Some(Box::new(FoldOp)),
            _ => None,
        }
    }
    
    /// Get all available unary operation names
    pub fn unary_op_names() -> Vec<&'static str> {
        vec![
            "+", "-", "abs", "sqrt", "sin", "cos", "tan", "exp", "log",
            "log2", "log10", "exp2", "floor", "ceil", "round", "trunc", "fract", "sign",
            "ampdb", "dbamp", "midicps", "cpsmidi", "distort", "softclip",
            "tanh", "sinh", "cosh"
        ]
    }
    
    /// Get all available binary operation names
    pub fn binary_op_names() -> Vec<&'static str> {
        vec![
            "+", "-", "*", "/", "%", "^", "<", "<=", ">", ">=", "==", "!=", 
            "min", "max", "clip2", "wrap2", "fold2"
        ]
    }
}

// Helper functions for applying operations to thread stack

/// Apply a unary operation to the top of the stack
pub fn apply_unary_op<T: UnaryOp>(thread: &mut Thread, op: &T) -> Result<(), SapfError> {
    let value = thread.pop()?;
    let result = op.apply_impl(thread, value)?;
    thread.push(result);
    Ok(())
}

/// Apply a binary operation to the top two values on the stack (b on top, a below)
pub fn apply_binary_op<T: BinaryOp>(thread: &mut Thread, op: &T) -> Result<(), SapfError> {
    let b = thread.pop()?;
    let a = thread.pop()?;
    let result = op.apply_impl(thread, a, b)?;
    thread.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::Thread;

    #[test]
    fn test_basic_arithmetic() {
        let mut thread = Thread::new();
        
        // Test addition
        thread.push(Value::Real(3.0));
        thread.push(Value::Real(4.0));
        apply_binary_op(&mut thread, &AddOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(7.0));
        
        // Test subtraction
        thread.push(Value::Real(10.0));
        thread.push(Value::Real(3.0));
        apply_binary_op(&mut thread, &SubOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(7.0));
        
        // Test multiplication
        thread.push(Value::Real(3.0));
        thread.push(Value::Real(4.0));
        apply_binary_op(&mut thread, &MulOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(12.0));
        
        // Test division
        thread.push(Value::Real(12.0));
        thread.push(Value::Real(3.0));
        apply_binary_op(&mut thread, &DivOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(4.0));
    }
    
    #[test]
    fn test_unary_operations() {
        let mut thread = Thread::new();
        
        // Test absolute value
        thread.push(Value::Real(-5.0));
        apply_unary_op(&mut thread, &AbsOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(5.0));
        
        // Test square root
        thread.push(Value::Real(16.0));
        apply_unary_op(&mut thread, &SqrtOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(4.0));
        
        // Test negation
        thread.push(Value::Real(5.0));
        apply_unary_op(&mut thread, &SubOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(-5.0));
    }
    
    #[test]
    fn test_comparison_operations() {
        let mut thread = Thread::new();
        
        // Test less than
        thread.push(Value::Real(3.0));
        thread.push(Value::Real(5.0));
        apply_binary_op(&mut thread, &LtOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(1.0));
        
        thread.push(Value::Real(5.0));
        thread.push(Value::Real(3.0));
        apply_binary_op(&mut thread, &LtOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(0.0));
        
        // Test equality
        thread.push(Value::Real(5.0));
        thread.push(Value::Real(5.0));
        apply_binary_op(&mut thread, &EqOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(1.0));
    }
    
    #[test]
    fn test_division_by_zero() {
        let div_op = DivOp;
        
        // Positive / 0 = +infinity
        assert_eq!(div_op.op(5.0, 0.0), f64::INFINITY);
        
        // Negative / 0 = -infinity
        assert_eq!(div_op.op(-5.0, 0.0), f64::NEG_INFINITY);
        
        // 0 / 0 = NaN
        assert!(div_op.op(0.0, 0.0).is_nan());
    }
    
    #[test]
    fn test_nil_propagation() {
        let mut thread = Thread::new();
        
        // Test that nil propagates through addition
        thread.push(Value::Real(5.0));
        thread.push(Value::Nil);
        apply_binary_op(&mut thread, &AddOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Nil);
        
        // Test that nil propagates through unary operations
        thread.push(Value::Nil);
        apply_unary_op(&mut thread, &AbsOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Nil);
    }
    
    #[test]
    fn test_operation_registry() {
        // Test unary operations
        assert!(MathOps::get_unary_op("abs").is_some());
        assert!(MathOps::get_unary_op("sqrt").is_some());
        assert!(MathOps::get_unary_op("nonexistent").is_none());
        
        // Test binary operations
        assert!(MathOps::get_binary_op("+").is_some());
        assert!(MathOps::get_binary_op("*").is_some());
        assert!(MathOps::get_binary_op("nonexistent").is_none());
    }
    
    #[test]
    fn test_identity_operations() {
        let add_op = AddOp;
        let mul_op = MulOp;
        
        // Addition identity is 0
        assert_eq!(add_op.identity(), Some(0.0));
        assert!(add_op.is_identity_with(0.0));
        assert!(!add_op.is_identity_with(1.0));
        
        // Multiplication identity is 1
        assert_eq!(mul_op.identity(), Some(1.0));
        assert!(mul_op.is_identity_with(1.0));
        assert!(!mul_op.is_identity_with(0.0));
    }
    
    #[test]
    fn test_mathematical_properties() {
        let add_op = AddOp;
        let mul_op = MulOp;
        let sub_op = SubOp;
        
        // Addition is commutative and associative
        assert!(add_op.is_commutative());
        assert!(add_op.is_associative());
        
        // Multiplication is commutative and associative
        assert!(mul_op.is_commutative());
        assert!(mul_op.is_associative());
        
        // Subtraction is neither
        assert!(!sub_op.is_commutative());
        assert!(!sub_op.is_associative());
    }
    
    #[test]
    fn test_extended_math_operations() {
        let mut thread = Thread::new();
        
        // Test log2
        thread.push(Value::Real(8.0));
        apply_unary_op(&mut thread, &Log2Op).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(3.0));
        
        // Test exp2
        thread.push(Value::Real(3.0));
        apply_unary_op(&mut thread, &Exp2Op).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(8.0));
        
        // Test floor
        thread.push(Value::Real(3.7));
        apply_unary_op(&mut thread, &FloorOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(3.0));
        
        // Test ceil
        thread.push(Value::Real(3.2));
        apply_unary_op(&mut thread, &CeilOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(4.0));
        
        // Test sign
        thread.push(Value::Real(-5.0));
        apply_unary_op(&mut thread, &SignOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(-1.0));
        
        thread.push(Value::Real(5.0));
        apply_unary_op(&mut thread, &SignOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(1.0));
        
        thread.push(Value::Real(0.0));
        apply_unary_op(&mut thread, &SignOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(0.0));
    }
    
    #[test]
    fn test_dsp_operations() {
        let mut thread = Thread::new();
        
        // Test ampdb - convert amplitude to decibels
        thread.push(Value::Real(1.0));
        apply_unary_op(&mut thread, &AmpDbOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(0.0)); // 1.0 amplitude = 0 dB
        
        // Test dbamp - convert decibels to amplitude
        thread.push(Value::Real(0.0));
        apply_unary_op(&mut thread, &DbAmpOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(1.0)); // 0 dB = 1.0 amplitude
        
        // Test midicps - MIDI note to frequency
        thread.push(Value::Real(69.0)); // A4
        apply_unary_op(&mut thread, &MidiCpsOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(440.0)); // 440 Hz
        
        // Test cpsmidi - frequency to MIDI note
        thread.push(Value::Real(440.0));
        apply_unary_op(&mut thread, &CpsMidiOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(69.0));
    }
    
    #[test]
    fn test_range_operations() {
        let mut thread = Thread::new();
        
        // Test clip2 - clip to range [-b, b]
        thread.push(Value::Real(5.0));
        thread.push(Value::Real(3.0));
        apply_binary_op(&mut thread, &ClipOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(3.0)); // clipped to 3
        
        thread.push(Value::Real(-5.0));
        thread.push(Value::Real(3.0));
        apply_binary_op(&mut thread, &ClipOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(-3.0)); // clipped to -3
        
        // Test distort
        thread.push(Value::Real(1.0));
        apply_unary_op(&mut thread, &DistortOp).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(0.5)); // 1 / (1 + 1) = 0.5
    }
    
    #[test]
    fn test_extended_registry() {
        // Test that all new operations are available
        assert!(MathOps::get_unary_op("log2").is_some());
        assert!(MathOps::get_unary_op("ampdb").is_some());
        assert!(MathOps::get_unary_op("midicps").is_some());
        assert!(MathOps::get_unary_op("distort").is_some());
        
        assert!(MathOps::get_binary_op("clip2").is_some());
        assert!(MathOps::get_binary_op("wrap2").is_some());
        assert!(MathOps::get_binary_op("fold2").is_some());
        
        // Test that operation lists include new operations
        let unary_names = MathOps::unary_op_names();
        assert!(unary_names.contains(&"log2"));
        assert!(unary_names.contains(&"ampdb"));
        assert!(unary_names.contains(&"midicps"));
        
        let binary_names = MathOps::binary_op_names();
        assert!(binary_names.contains(&"clip2"));
        assert!(binary_names.contains(&"wrap2"));
        assert!(binary_names.contains(&"fold2"));
    }
}
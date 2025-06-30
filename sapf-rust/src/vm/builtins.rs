//! Built-in function registration system for SAPF VM
//!
//! This module provides the registration system for built-in functions,
//! including core operations and mathematical functions.

use crate::core::{SapfError, Value, Object};
use crate::core::function::Primitive;
use crate::core::math_ops::*;
use crate::core::core_ops;
use crate::core::random_ops;
// use crate::audio::audio_ops;
use crate::vm::{VM, Thread};
use std::sync::Arc;

/// Function signature for built-in operations that work directly with Thread
pub type BuiltinFn = fn(&mut Thread, &[Value]) -> Result<Vec<Value>, SapfError>;

// Core operation wrapper functions
fn clear_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::clear(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn cleard_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::cleard(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn stack_depth_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::stack_depth(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn ba_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::ba(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn bac_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::bac(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn cab_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::cab(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn bca_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::bca(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn cba_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::cba(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn aa_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::aa(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn aaa_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::aaa(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn aba_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::aba(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn bab_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::bab(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn aab_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::aab(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn aabb_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::aabb(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn abab_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::abab(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn nip_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::nip(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn pop_op_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::pop_op(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn equals_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::equals(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn less_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::less(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn greater_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::greater(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn not_op_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::not_op(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn pr_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::pr(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn cr_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::cr(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn sp_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::sp(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn tab_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::tab(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn prstk_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::prstk(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn type_op_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::type_op(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn sr_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::sr(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn nyq_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::nyq(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn isr_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::isr(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn rps_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::rps(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

fn inyq_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    let results = core_ops::inyq(thread, &[])?;
    for result in results {
        thread.push(result);
    }
    Ok(())
}

// Wrapper functions for specific math operations

// Unary operation wrappers
fn add_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &AddOp)
}

fn sub_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &SubOp)
}

fn mul_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &MulOp)
}

fn div_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &DivOp)
}

fn mod_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &ModOp)
}

fn pow_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &PowOp)
}

fn abs_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &AbsOp)
}

fn sqrt_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &SqrtOp)
}

fn sin_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &SinOp)
}

fn cos_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &CosOp)
}

fn tan_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &TanOp)
}

fn exp_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &ExpOp)
}

fn log_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &LogOp)
}

fn log2_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &Log2Op)
}

fn log10_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &Log10Op)
}

fn exp2_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &Exp2Op)
}

fn floor_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &FloorOp)
}

fn ceil_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &CeilOp)
}

fn round_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &RoundOp)
}

fn trunc_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &TruncOp)
}

fn fract_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &FractOp)
}

fn sign_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &SignOp)
}

fn ampdb_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &AmpDbOp)
}

fn dbamp_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &DbAmpOp)
}

fn midicps_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &MidiCpsOp)
}

fn cpsmidi_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &CpsMidiOp)
}

fn distort_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &DistortOp)
}

fn softclip_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &SoftClipOp)
}

fn tanh_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &TanhOp)
}

fn sinh_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &SinhOp)
}

fn cosh_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_unary_op(thread, &CoshOp)
}

fn le_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &LeOp)
}

fn ge_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &GeOp)
}

fn ne_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &NeOp)
}

fn min_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &MinOp)
}

fn max_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &MaxOp)
}

fn clip_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &ClipOp)
}

fn wrap_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &WrapOp)
}

fn fold_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    apply_binary_op(thread, &FoldOp)
}

/// Register all core built-in functions with the VM
pub fn register_core_builtins(vm: &VM) -> Result<(), SapfError> {
    // Stack operations
    vm.def_by_name("clear", Value::from_object(Arc::new(Primitive::new(
        clear_prim, Value::Real(0.0), "clear", "Clear the stack", 0, 0
    ))))?;
    
    vm.def_by_name("cleard", Value::from_object(Arc::new(Primitive::new(
        cleard_prim, Value::Real(0.0), "cleard", "Clear all but top of stack", 1, 1
    ))))?;
    
    vm.def_by_name("depth", Value::from_object(Arc::new(Primitive::new(
        stack_depth_prim, Value::Real(0.0), "depth", "Get stack depth", 0, 1
    ))))?;
    
    vm.def_by_name("ba", Value::from_object(Arc::new(Primitive::new(
        ba_prim, Value::Real(0.0), "ba", "Swap top two stack items", 2, 2
    ))))?;
    
    vm.def_by_name("bac", Value::from_object(Arc::new(Primitive::new(
        bac_prim, Value::Real(0.0), "bac", "Rotate top three items: a b c -> b a c", 3, 3
    ))))?;
    
    vm.def_by_name("cab", Value::from_object(Arc::new(Primitive::new(
        cab_prim, Value::Real(0.0), "cab", "Rotate top three items: a b c -> c a b", 3, 3
    ))))?;
    
    vm.def_by_name("bca", Value::from_object(Arc::new(Primitive::new(
        bca_prim, Value::Real(0.0), "bca", "Rotate top three items: a b c -> b c a", 3, 3
    ))))?;
    
    vm.def_by_name("cba", Value::from_object(Arc::new(Primitive::new(
        cba_prim, Value::Real(0.0), "cba", "Reverse top three items: a b c -> c b a", 3, 3
    ))))?;
    
    vm.def_by_name("aa", Value::from_object(Arc::new(Primitive::new(
        aa_prim, Value::Real(0.0), "aa", "Duplicate top stack item", 1, 2
    ))))?;
    
    vm.def_by_name("dup", Value::from_object(Arc::new(Primitive::new(
        aa_prim, Value::Real(0.0), "dup", "Duplicate top stack item", 1, 2
    ))))?;
    
    vm.def_by_name("aaa", Value::from_object(Arc::new(Primitive::new(
        aaa_prim, Value::Real(0.0), "aaa", "Duplicate top item twice", 1, 3
    ))))?;
    
    vm.def_by_name("aba", Value::from_object(Arc::new(Primitive::new(
        aba_prim, Value::Real(0.0), "aba", "Insert copy of second item on top", 2, 3
    ))))?;
    
    vm.def_by_name("bab", Value::from_object(Arc::new(Primitive::new(
        bab_prim, Value::Real(0.0), "bab", "Insert copy of top item in second position", 2, 3
    ))))?;
    
    vm.def_by_name("aab", Value::from_object(Arc::new(Primitive::new(
        aab_prim, Value::Real(0.0), "aab", "Copy second item to top", 2, 3
    ))))?;
    
    vm.def_by_name("aabb", Value::from_object(Arc::new(Primitive::new(
        aabb_prim, Value::Real(0.0), "aabb", "Duplicate top two items", 2, 4
    ))))?;
    
    vm.def_by_name("abab", Value::from_object(Arc::new(Primitive::new(
        abab_prim, Value::Real(0.0), "abab", "Interleave top two items", 2, 4
    ))))?;
    
    vm.def_by_name("nip", Value::from_object(Arc::new(Primitive::new(
        nip_prim, Value::Real(0.0), "nip", "Remove second stack item", 2, 1
    ))))?;
    
    vm.def_by_name("pop", Value::from_object(Arc::new(Primitive::new(
        pop_op_prim, Value::Real(0.0), "pop", "Remove top stack item", 1, 0
    ))))?;
    
    // Comparison operations
    vm.def_by_name("==", Value::from_object(Arc::new(Primitive::new(
        equals_prim, Value::Real(0.0), "==", "Test equality", 2, 1
    ))))?;
    
    vm.def_by_name("<", Value::from_object(Arc::new(Primitive::new(
        less_prim, Value::Real(0.0), "<", "Test less than", 2, 1
    ))))?;
    
    vm.def_by_name(">", Value::from_object(Arc::new(Primitive::new(
        greater_prim, Value::Real(0.0), ">", "Test greater than", 2, 1
    ))))?;
    
    vm.def_by_name("not", Value::from_object(Arc::new(Primitive::new(
        not_op_prim, Value::Real(0.0), "not", "Logical not", 1, 1
    ))))?;
    
    // Print operations
    vm.def_by_name("pr", Value::from_object(Arc::new(Primitive::new(
        pr_prim, Value::Real(0.0), "pr", "Print top of stack", 1, 1
    ))))?;
    
    vm.def_by_name("cr", Value::from_object(Arc::new(Primitive::new(
        cr_prim, Value::Real(0.0), "cr", "Print carriage return", 0, 0
    ))))?;
    
    vm.def_by_name("sp", Value::from_object(Arc::new(Primitive::new(
        sp_prim, Value::Real(0.0), "sp", "Print space", 0, 0
    ))))?;
    
    vm.def_by_name("tab", Value::from_object(Arc::new(Primitive::new(
        tab_prim, Value::Real(0.0), "tab", "Print tab", 0, 0
    ))))?;
    
    vm.def_by_name("prstk", Value::from_object(Arc::new(Primitive::new(
        prstk_prim, Value::Real(0.0), "prstk", "Print stack contents", 0, 0
    ))))?;
    
    // Type operation
    vm.def_by_name("type", Value::from_object(Arc::new(Primitive::new(
        type_op_prim, Value::Real(0.0), "type", "Get type of top stack item", 1, 1
    ))))?;
    
    // Audio rate operations
    vm.def_by_name("sr", Value::from_object(Arc::new(Primitive::new(
        sr_prim, Value::Real(0.0), "sr", "Get sample rate", 0, 1
    ))))?;
    
    vm.def_by_name("nyq", Value::from_object(Arc::new(Primitive::new(
        nyq_prim, Value::Real(0.0), "nyq", "Get Nyquist frequency", 0, 1
    ))))?;
    
    vm.def_by_name("isr", Value::from_object(Arc::new(Primitive::new(
        isr_prim, Value::Real(0.0), "isr", "Get inverse sample rate", 0, 1
    ))))?;
    
    vm.def_by_name("rps", Value::from_object(Arc::new(Primitive::new(
        rps_prim, Value::Real(0.0), "rps", "Get radians per sample", 0, 1
    ))))?;
    
    vm.def_by_name("inyq", Value::from_object(Arc::new(Primitive::new(
        inyq_prim, Value::Real(0.0), "inyq", "Get inverse Nyquist frequency", 0, 1
    ))))?;
    
    Ok(())
}

/// Register mathematical built-in functions with the VM
pub fn register_math_builtins(vm: &VM) -> Result<(), SapfError> {
    // Basic arithmetic (these will be handled by binary operators)
    vm.def_by_name("+", Value::from_object(Arc::new(Primitive::new(
        add_prim, Value::Real(0.0), "+", "Addition", 2, 1
    ))))?;
    
    vm.def_by_name("-", Value::from_object(Arc::new(Primitive::new(
        sub_prim, Value::Real(0.0), "-", "Subtraction", 2, 1
    ))))?;
    
    vm.def_by_name("*", Value::from_object(Arc::new(Primitive::new(
        mul_prim, Value::Real(0.0), "*", "Multiplication", 2, 1
    ))))?;
    
    vm.def_by_name("/", Value::from_object(Arc::new(Primitive::new(
        div_prim, Value::Real(0.0), "/", "Division", 2, 1
    ))))?;
    
    vm.def_by_name("%", Value::from_object(Arc::new(Primitive::new(
        mod_prim, Value::Real(0.0), "%", "Modulo", 2, 1
    ))))?;
    
    vm.def_by_name("**", Value::from_object(Arc::new(Primitive::new(
        pow_prim, Value::Real(0.0), "**", "Power/Exponentiation", 2, 1
    ))))?;
    
    // Unary mathematical functions
    vm.def_by_name("abs", Value::from_object(Arc::new(Primitive::new(
        abs_prim, Value::Real(0.0), "abs", "Absolute value", 1, 1
    ))))?;
    
    vm.def_by_name("sqrt", Value::from_object(Arc::new(Primitive::new(
        sqrt_prim, Value::Real(0.0), "sqrt", "Square root", 1, 1
    ))))?;
    
    vm.def_by_name("sin", Value::from_object(Arc::new(Primitive::new(
        sin_prim, Value::Real(0.0), "sin", "Sine", 1, 1
    ))))?;
    
    vm.def_by_name("cos", Value::from_object(Arc::new(Primitive::new(
        cos_prim, Value::Real(0.0), "cos", "Cosine", 1, 1
    ))))?;
    
    vm.def_by_name("tan", Value::from_object(Arc::new(Primitive::new(
        tan_prim, Value::Real(0.0), "tan", "Tangent", 1, 1
    ))))?;
    
    vm.def_by_name("exp", Value::from_object(Arc::new(Primitive::new(
        exp_prim, Value::Real(0.0), "exp", "Exponential (e^x)", 1, 1
    ))))?;
    
    vm.def_by_name("log", Value::from_object(Arc::new(Primitive::new(
        log_prim, Value::Real(0.0), "log", "Natural logarithm", 1, 1
    ))))?;
    
    vm.def_by_name("log2", Value::from_object(Arc::new(Primitive::new(
        log2_prim, Value::Real(0.0), "log2", "Base-2 logarithm", 1, 1
    ))))?;
    
    vm.def_by_name("log10", Value::from_object(Arc::new(Primitive::new(
        log10_prim, Value::Real(0.0), "log10", "Base-10 logarithm", 1, 1
    ))))?;
    
    vm.def_by_name("exp2", Value::from_object(Arc::new(Primitive::new(
        exp2_prim, Value::Real(0.0), "exp2", "Base-2 exponential (2^x)", 1, 1
    ))))?;
    
    vm.def_by_name("floor", Value::from_object(Arc::new(Primitive::new(
        floor_prim, Value::Real(0.0), "floor", "Floor function", 1, 1
    ))))?;
    
    vm.def_by_name("ceil", Value::from_object(Arc::new(Primitive::new(
        ceil_prim, Value::Real(0.0), "ceil", "Ceiling function", 1, 1
    ))))?;
    
    vm.def_by_name("round", Value::from_object(Arc::new(Primitive::new(
        round_prim, Value::Real(0.0), "round", "Round to nearest integer", 1, 1
    ))))?;
    
    vm.def_by_name("trunc", Value::from_object(Arc::new(Primitive::new(
        trunc_prim, Value::Real(0.0), "trunc", "Truncate to integer", 1, 1
    ))))?;
    
    vm.def_by_name("fract", Value::from_object(Arc::new(Primitive::new(
        fract_prim, Value::Real(0.0), "fract", "Fractional part", 1, 1
    ))))?;
    
    vm.def_by_name("sign", Value::from_object(Arc::new(Primitive::new(
        sign_prim, Value::Real(0.0), "sign", "Sign function", 1, 1
    ))))?;
    
    // DSP-specific functions
    vm.def_by_name("ampdb", Value::from_object(Arc::new(Primitive::new(
        ampdb_prim, Value::Real(0.0), "ampdb", "Convert amplitude to dB", 1, 1
    ))))?;
    
    vm.def_by_name("dbamp", Value::from_object(Arc::new(Primitive::new(
        dbamp_prim, Value::Real(0.0), "dbamp", "Convert dB to amplitude", 1, 1
    ))))?;
    
    vm.def_by_name("midicps", Value::from_object(Arc::new(Primitive::new(
        midicps_prim, Value::Real(0.0), "midicps", "Convert MIDI note to frequency", 1, 1
    ))))?;
    
    vm.def_by_name("cpsmidi", Value::from_object(Arc::new(Primitive::new(
        cpsmidi_prim, Value::Real(0.0), "cpsmidi", "Convert frequency to MIDI note", 1, 1
    ))))?;
    
    vm.def_by_name("distort", Value::from_object(Arc::new(Primitive::new(
        distort_prim, Value::Real(0.0), "distort", "Distortion function", 1, 1
    ))))?;
    
    vm.def_by_name("softclip", Value::from_object(Arc::new(Primitive::new(
        softclip_prim, Value::Real(0.0), "softclip", "Soft clipping", 1, 1
    ))))?;
    
    // Hyperbolic functions
    vm.def_by_name("tanh", Value::from_object(Arc::new(Primitive::new(
        tanh_prim, Value::Real(0.0), "tanh", "Hyperbolic tangent", 1, 1
    ))))?;
    
    vm.def_by_name("sinh", Value::from_object(Arc::new(Primitive::new(
        sinh_prim, Value::Real(0.0), "sinh", "Hyperbolic sine", 1, 1
    ))))?;
    
    vm.def_by_name("cosh", Value::from_object(Arc::new(Primitive::new(
        cosh_prim, Value::Real(0.0), "cosh", "Hyperbolic cosine", 1, 1
    ))))?;
    
    // Binary comparison and utility functions
    vm.def_by_name("<=", Value::from_object(Arc::new(Primitive::new(
        le_prim, Value::Real(0.0), "<=", "Less than or equal", 2, 1
    ))))?;
    
    vm.def_by_name(">=", Value::from_object(Arc::new(Primitive::new(
        ge_prim, Value::Real(0.0), ">=", "Greater than or equal", 2, 1
    ))))?;
    
    vm.def_by_name("!=", Value::from_object(Arc::new(Primitive::new(
        ne_prim, Value::Real(0.0), "!=", "Not equal", 2, 1
    ))))?;
    
    vm.def_by_name("min", Value::from_object(Arc::new(Primitive::new(
        min_prim, Value::Real(0.0), "min", "Minimum of two values", 2, 1
    ))))?;
    
    vm.def_by_name("max", Value::from_object(Arc::new(Primitive::new(
        max_prim, Value::Real(0.0), "max", "Maximum of two values", 2, 1
    ))))?;
    
    vm.def_by_name("clip", Value::from_object(Arc::new(Primitive::new(
        clip_prim, Value::Real(0.0), "clip", "Clip value to range [-arg, +arg]", 2, 1
    ))))?;
    
    vm.def_by_name("wrap", Value::from_object(Arc::new(Primitive::new(
        wrap_prim, Value::Real(0.0), "wrap", "Wrap value to range [-arg, +arg]", 2, 1
    ))))?;
    
    vm.def_by_name("fold", Value::from_object(Arc::new(Primitive::new(
        fold_prim, Value::Real(0.0), "fold", "Fold value to range [-arg, +arg]", 2, 1
    ))))?;
    
    Ok(())
}

/// Register all built-in functions with the VM
pub fn register_all_builtins(vm: &VM) -> Result<(), SapfError> {
    register_core_builtins(vm)?;
    register_math_builtins(vm)?;
    random_ops::register_random_builtins(vm)?;
    // Temporarily disabled until cpal threading issues are resolved
    // audio_ops::register_audio_builtins(vm)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;
    
    #[test]
    fn test_register_core_builtins() {
        let vm = VM::new();
        register_core_builtins(&vm).expect("Should register core builtins");
        
        // Test that some functions are registered
        assert!(vm.lookup_by_name("clear").is_some());
        assert!(vm.lookup_by_name("aa").is_some());
        assert!(vm.lookup_by_name("dup").is_some());
        assert!(vm.lookup_by_name("ba").is_some());
        assert!(vm.lookup_by_name("==").is_some());
        assert!(vm.lookup_by_name("pr").is_some());
    }
    
    #[test]
    fn test_register_math_builtins() {
        let vm = VM::new();
        register_math_builtins(&vm).expect("Should register math builtins");
        
        // Test that some math functions are registered
        assert!(vm.lookup_by_name("+").is_some());
        assert!(vm.lookup_by_name("-").is_some());
        assert!(vm.lookup_by_name("*").is_some());
        assert!(vm.lookup_by_name("/").is_some());
        assert!(vm.lookup_by_name("sin").is_some());
        assert!(vm.lookup_by_name("cos").is_some());
        assert!(vm.lookup_by_name("sqrt").is_some());
    }
    
    #[test]
    fn test_register_all_builtins() {
        let vm = VM::new();
        register_all_builtins(&vm).expect("Should register all builtins");
        
        // Test that functions from both modules are available
        assert!(vm.lookup_by_name("clear").is_some());
        assert!(vm.lookup_by_name("dup").is_some());
        assert!(vm.lookup_by_name("+").is_some());
        assert!(vm.lookup_by_name("sin").is_some());
        assert!(vm.lookup_by_name("prstk").is_some());
    }
}
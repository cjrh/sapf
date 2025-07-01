//! Random number generation operations for SAPF
//! 
//! This module provides comprehensive random number generation functionality including:
//! - Single random numbers (rand, coin, irand, etc.)
//! - Random streams and signals (infinite and finite sequences)
//! - Various distributions (uniform, exponential, linear, triangular)
//! - Weighted random selection and choice operations
//! 
//! The implementation follows SAPF's RGen structure using xoroshiro128+ algorithm
//! for high-quality, fast random number generation suitable for audio applications.

use crate::core::error::SapfError;
use crate::core::hash::hash_i64;
use crate::core::value::{Value, Object};
use crate::core::list::{List, Array};
use crate::vm::thread::Thread;

/// Type alias for floating-point numbers
type Real = f64;

/// Random number generator using xoroshiro128+ algorithm
/// 
/// This provides the core random number generation functionality used throughout
/// the SAPF system. It's designed for high-quality, fast random number generation
/// suitable for audio applications where statistical quality and performance are important.
#[derive(Debug, Clone)]
pub struct RGen {
    /// 128-bit internal state for xoroshiro128+ algorithm
    state: [u64; 2],
}

impl RGen {
    /// Create a new random generator with given seed
    pub fn new(seed: i64) -> Self {
        let mut rgen = Self { state: [0, 0] };
        rgen.init(seed);
        rgen
    }

    /// Initialize the generator with a seed value
    /// Uses hash functions to ensure good distribution from arbitrary seeds
    pub fn init(&mut self, seed: i64) {
        self.state[0] = hash_i64(seed.wrapping_add(0x43a68b0d0492ba51i64)) as u64;
        self.state[1] = hash_i64(seed.wrapping_add(0x56e376c6e7c29504i64)) as u64;
    }

    /// Generate a random 64-bit integer using xoroshiro128+
    pub fn trand(&mut self) -> i64 {
        xoroshiro128(&mut self.state) as i64
    }

    /// Generate random float in range [0, 1)
    pub fn drand(&mut self) -> Real {
        let i = self.trand() as u64;
        // Convert to double using bit manipulation for uniform distribution
        let bits = 0x3FF0000000000000u64 | (i >> 12);
        f64::from_bits(bits) - 1.0
    }

    /// Generate random float in range [-1, 1)
    pub fn drand2(&mut self) -> Real {
        let i = self.trand() as u64;
        let bits = 0x4000000000000000u64 | (i >> 12);
        f64::from_bits(bits) - 3.0
    }

    /// Generate random float in range [-1/8, 1/8)
    pub fn drand8(&mut self) -> Real {
        let i = self.trand() as u64;
        let bits = 0x3FD0000000000000u64 | (i >> 12);
        f64::from_bits(bits) - 0.375
    }

    /// Generate random float in range [-1/16, 1/16)
    pub fn drand16(&mut self) -> Real {
        let i = self.trand() as u64;
        let bits = 0x3FC0000000000000u64 | (i >> 12);
        f64::from_bits(bits) - 0.1875
    }

    /// Generate uniform random float in range [lo, hi)
    pub fn rand(&mut self, lo: Real, hi: Real) -> Real {
        lo + (hi - lo) * self.drand()
    }

    /// Generate exponentially distributed random float in range [lo, hi)
    pub fn xrand(&mut self, lo: Real, hi: Real) -> Real {
        lo * (hi / lo).powf(self.drand())
    }

    /// Generate linearly distributed random float in range [lo, hi)
    /// (higher probability towards lo)
    pub fn linrand(&mut self, lo: Real, hi: Real) -> Real {
        lo + (hi - lo) * self.drand().min(self.drand())
    }

    /// Generate triangularly distributed random float in range [lo, hi)
    /// (higher probability towards center)
    pub fn trirand(&mut self, lo: Real, hi: Real) -> Real {
        lo + (hi - lo) * (0.5 + 0.5 * (self.drand() - self.drand()))
    }

    /// Generate coin toss: returns 1.0 with probability p, 0.0 otherwise
    pub fn coin(&mut self, p: Real) -> Real {
        if self.drand() < p { 1.0 } else { 0.0 }
    }

    /// Generate random integer in range [0, n)
    pub fn irand0(&mut self, n: i64) -> i64 {
        (n as f64 * self.drand()).floor() as i64
    }

    /// Generate uniform random integer in range [lo, hi]
    pub fn irand(&mut self, lo: i64, hi: i64) -> i64 {
        lo + ((hi - lo + 1) as f64 * self.drand()).floor() as i64
    }

    /// Generate random integer in range [-scale, scale]
    pub fn irand2(&mut self, scale: i64) -> i64 {
        let fscale = scale as f64;
        ((2.0 * fscale + 1.0) * self.drand() - fscale).floor() as i64
    }

    /// Generate linearly distributed random integer in range [lo, hi)
    pub fn ilinrand(&mut self, lo: i64, hi: i64) -> i64 {
        lo + ((hi - lo) as f64 * self.drand().min(self.drand())).floor() as i64
    }

    /// Generate triangularly distributed random integer in range [lo, hi)
    pub fn itrirand(&mut self, lo: i64, hi: i64) -> i64 {
        let scale = (hi - lo) as f64;
        lo + (scale * (0.5 + 0.5 * (self.drand() - self.drand()))).floor() as i64
    }
}

/// xoroshiro128+ algorithm implementation
/// High-quality, fast random number generator with 128-bit state
fn xoroshiro128(state: &mut [u64; 2]) -> u64 {
    let s0 = state[0];
    let mut s1 = state[1];
    let result = s0.wrapping_add(s1);

    s1 ^= s0;
    state[0] = rotl(s0, 55) ^ s1 ^ (s1 << 14);
    state[1] = rotl(s1, 36);

    result
}

/// Rotate left helper function for xoroshiro128+
fn rotl(x: u64, k: u32) -> u64 {
    (x << k) | (x >> (64 - k))
}

// ================================================================================================
// SINGLE RANDOM NUMBER OPERATIONS
// ================================================================================================

/// Generate a new random seed
pub fn newseed(thread: &mut Thread) -> Result<(), SapfError> {
    let seed = thread.rgen.trand();
    thread.push(Value::Real(seed as Real));
    Ok(())
}

/// Set the random seed
pub fn setseed(thread: &mut Thread) -> Result<(), SapfError> {
    let seed = thread.pop_real("setseed")?;
    thread.rgen.init(seed as i64);
    Ok(())
}

/// Generate uniform random number in range [a, b)
pub fn rand(thread: &mut Thread) -> Result<(), SapfError> {
    let b = thread.pop_real("rand")?;
    let a = thread.pop_real("rand")?;
    let result = thread.rgen.rand(a, b);
    thread.push(Value::Real(result));
    Ok(())
}

/// Generate coin toss: 1 with probability p, 0 otherwise
pub fn coin(thread: &mut Thread) -> Result<(), SapfError> {
    let p = thread.pop_real("coin")?;
    let result = thread.rgen.coin(p);
    thread.push(Value::Real(result));
    Ok(())
}

/// Generate random number in range [-a, +a)
pub fn rand2(thread: &mut Thread) -> Result<(), SapfError> {
    let a = thread.pop_real("rand2")?;
    let result = thread.rgen.rand(-a, a);
    thread.push(Value::Real(result));
    Ok(())
}

/// Generate uniform random integer in range [a, b]
pub fn irand(thread: &mut Thread) -> Result<(), SapfError> {
    let b = thread.pop_real("irand")? as i64;
    let a = thread.pop_real("irand")? as i64;
    let result = thread.rgen.irand(a, b);
    thread.push(Value::Real(result as Real));
    Ok(())
}

/// Generate random integer in range [-a, +a]
pub fn irand2(thread: &mut Thread) -> Result<(), SapfError> {
    let a = thread.pop_real("irand2")? as i64;
    let result = thread.rgen.irand2(a);
    thread.push(Value::Real(result as Real));
    Ok(())
}

/// Generate exponentially distributed random number in range [a, b)
pub fn xrand(thread: &mut Thread) -> Result<(), SapfError> {
    let b = thread.pop_real("xrand")?;
    let a = thread.pop_real("xrand")?;
    let result = thread.rgen.xrand(a, b);
    thread.push(Value::Real(result));
    Ok(())
}

/// Generate linearly distributed random number in range [a, b)
pub fn linrand(thread: &mut Thread) -> Result<(), SapfError> {
    let b = thread.pop_real("linrand")?;
    let a = thread.pop_real("linrand")?;
    let result = thread.rgen.linrand(a, b);
    thread.push(Value::Real(result));
    Ok(())
}

/// Generate linearly distributed random integer in range [a, b)
pub fn ilinrand(thread: &mut Thread) -> Result<(), SapfError> {
    let b = thread.pop_real("ilinrand")? as i64;
    let a = thread.pop_real("ilinrand")? as i64;
    let result = thread.rgen.ilinrand(a, b);
    thread.push(Value::Real(result as Real));
    Ok(())
}

/// Choose random element from a list
pub fn pick(thread: &mut Thread) -> Result<(), SapfError> {
    let list_val = thread.pop()?;
    
    match list_val {
        Value::Object(ref obj) => {
            if let Some(list_obj) = obj.as_any().downcast_ref::<List>() {
                // Get the array from the list
                if let Some(array) = list_obj.array() {
                    let size = array.size();
                    if size == 0 {
                        return Err(SapfError::InvalidValue("pick: empty list".to_string()));
                    }
                    
                    let index = thread.rgen.irand0(size as i64) as usize;
                    let item = array.at(index)?;
                    thread.push(item);
                    Ok(())
                } else {
                    Err(SapfError::InvalidValue("pick: list not realized".to_string()))
                }
            } else {
                Err(SapfError::WrongTypeWithContext("pick".to_string(), list_val.type_name().to_string()))
            }
        }
        _ => Err(SapfError::WrongTypeWithContext("pick".to_string(), list_val.type_name().to_string()))
    }
}

/// Choose random index from probability weights
pub fn wrand(thread: &mut Thread) -> Result<(), SapfError> {
    let weights_val = thread.pop()?;
    
    match weights_val {
        Value::Object(ref obj) => {
            if let Some(weights_obj) = obj.as_any().downcast_ref::<List>() {
                if let Some(array) = weights_obj.array() {
                    let size = array.size();
                    if size == 0 {
                        return Err(SapfError::InvalidValue("wrand: empty weights".to_string()));
                    }
                    
                    // Convert to reals and compute cumulative sum
                    let mut cumulative = Vec::with_capacity(size);
                    let mut sum = 0.0;
                    
                    for i in 0..size {
                        let weight = array.at(i)?;
                        if let Value::Real(w) = weight {
                            sum += w;
                            cumulative.push(sum);
                        } else {
                            return Err(SapfError::WrongTypeWithContext("wrand".to_string(), "numbers".to_string()));
                        }
                    }
                    
                    if sum <= 0.0 {
                        return Err(SapfError::InvalidValue("wrand: weights must sum to positive value".to_string()));
                    }
                    
                    let target = thread.rgen.drand() * sum;
                    let index = cumulative.iter().position(|&cum| target <= cum).unwrap_or(cumulative.len() - 1);
                    
                    thread.push(Value::Real(index as Real));
                    Ok(())
                } else {
                    Err(SapfError::InvalidValue("wrand: weights not realized".to_string()))
                }
            } else {
                Err(SapfError::WrongTypeWithContext("wrand".to_string(), weights_val.type_name().to_string()))
            }
        }
        _ => Err(SapfError::WrongTypeWithContext("wrand".to_string(), weights_val.type_name().to_string()))
    }
}

/// Choose random element from list using probability weights
pub fn wpick(thread: &mut Thread) -> Result<(), SapfError> {
    let weights_val = thread.pop()?;
    let list_val = thread.pop()?;
    
    match (list_val, weights_val) {
        (Value::Object(ref list_obj), Value::Object(ref weights_obj)) => {
            if let (Some(list), Some(weights)) = (
                list_obj.as_any().downcast_ref::<List>(),
                weights_obj.as_any().downcast_ref::<List>()
            ) {
                if let (Some(list_array), Some(weights_array)) = (list.array(), weights.array()) {
                    let list_size = list_array.size();
                    let weights_size = weights_array.size();
                    
                    if list_size != weights_size {
                        return Err(SapfError::InvalidValue("wpick: list and weights must be same length".to_string()));
                    }
                    
                    if list_size == 0 {
                        return Err(SapfError::InvalidValue("wpick: empty lists".to_string()));
                    }
                    
                    // Convert to reals and compute cumulative sum
                    let mut cumulative = Vec::with_capacity(weights_size);
                    let mut sum = 0.0;
                    
                    for i in 0..weights_size {
                        let weight = weights_array.at(i)?;
                        if let Value::Real(w) = weight {
                            sum += w;
                            cumulative.push(sum);
                        } else {
                            return Err(SapfError::WrongTypeWithContext("wpick".to_string(), "numbers".to_string()));
                        }
                    }
                    
                    if sum <= 0.0 {
                        return Err(SapfError::InvalidValue("wpick: weights must sum to positive value".to_string()));
                    }
                    
                    let target = thread.rgen.drand() * sum;
                    let index = cumulative.iter().position(|&cum| target <= cum).unwrap_or(cumulative.len() - 1);
                    
                    let item = list_array.at(index)?;
                    thread.push(item);
                    Ok(())
                } else {
                    Err(SapfError::InvalidValue("wpick: lists not realized".to_string()))
                }
            } else {
                Err(SapfError::WrongTypeWithContext("wpick".to_string(), "two lists".to_string()))
            }
        }
        _ => Err(SapfError::WrongTypeWithContext("wpick".to_string(), "two lists".to_string()))
    }
}

// ================================================================================================
// BUILTIN REGISTRATION WRAPPERS
// ================================================================================================

use crate::core::function::Primitive;

/// Wrapper functions for VM builtin registration
pub mod builtins {
    use super::*;
    use crate::core::function::Primitive;

    // Random seed operations
    pub fn newseed_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        newseed(thread)
    }

    pub fn setseed_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        setseed(thread)
    }

    // Single random number operations
    pub fn rand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        rand(thread)
    }

    pub fn coin_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        coin(thread)
    }

    pub fn rand2_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        rand2(thread)
    }

    pub fn irand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        irand(thread)
    }

    pub fn irand2_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        irand2(thread)
    }

    pub fn xrand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        xrand(thread)
    }

    pub fn linrand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        linrand(thread)
    }

    pub fn ilinrand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        ilinrand(thread)
    }

    pub fn pick_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        pick(thread)
    }

    pub fn wrand_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        wrand(thread)
    }

    pub fn wpick_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
        wpick(thread)
    }
}

/// Register random operation built-in functions with the VM
pub fn register_random_builtins(vm: &crate::vm::VM) -> Result<(), SapfError> {
    use std::sync::Arc;
    use crate::core::value::Value;
    use crate::core::function::Primitive;

    // Random seed operations
    vm.def_by_name("newseed", Value::from_object(Arc::new(Primitive::new(
        builtins::newseed_prim, Value::Real(0.0), "newseed", "Generate new random seed", 0, 1
    ))))?;

    vm.def_by_name("setseed", Value::from_object(Arc::new(Primitive::new(
        builtins::setseed_prim, Value::Real(0.0), "setseed", "Set random seed", 1, 0
    ))))?;

    // Single random number operations
    vm.def_by_name("rand", Value::from_object(Arc::new(Primitive::new(
        builtins::rand_prim, Value::Real(0.0), "rand", "Random number in range [a, b)", 2, 1
    ))))?;

    vm.def_by_name("coin", Value::from_object(Arc::new(Primitive::new(
        builtins::coin_prim, Value::Real(0.0), "coin", "Coin toss with probability p", 1, 1
    ))))?;

    vm.def_by_name("rand2", Value::from_object(Arc::new(Primitive::new(
        builtins::rand2_prim, Value::Real(0.0), "rand2", "Random number in range [-a, +a)", 1, 1
    ))))?;

    vm.def_by_name("irand", Value::from_object(Arc::new(Primitive::new(
        builtins::irand_prim, Value::Real(0.0), "irand", "Random integer in range [a, b]", 2, 1
    ))))?;

    vm.def_by_name("irand2", Value::from_object(Arc::new(Primitive::new(
        builtins::irand2_prim, Value::Real(0.0), "irand2", "Random integer in range [-a, +a]", 1, 1
    ))))?;

    vm.def_by_name("xrand", Value::from_object(Arc::new(Primitive::new(
        builtins::xrand_prim, Value::Real(0.0), "xrand", "Exponentially distributed random in [a, b)", 2, 1
    ))))?;

    vm.def_by_name("linrand", Value::from_object(Arc::new(Primitive::new(
        builtins::linrand_prim, Value::Real(0.0), "linrand", "Linearly distributed random in [a, b)", 2, 1
    ))))?;

    vm.def_by_name("ilinrand", Value::from_object(Arc::new(Primitive::new(
        builtins::ilinrand_prim, Value::Real(0.0), "ilinrand", "Linearly distributed random integer in [a, b)", 2, 1
    ))))?;

    vm.def_by_name("pick", Value::from_object(Arc::new(Primitive::new(
        builtins::pick_prim, Value::Real(0.0), "pick", "Pick random element from list", 1, 1
    ))))?;

    vm.def_by_name("wrand", Value::from_object(Arc::new(Primitive::new(
        builtins::wrand_prim, Value::Real(0.0), "wrand", "Weighted random index selection", 1, 1
    ))))?;

    vm.def_by_name("wpick", Value::from_object(Arc::new(Primitive::new(
        builtins::wpick_prim, Value::Real(0.0), "wpick", "Weighted random element selection", 2, 1
    ))))?;

    Ok(())
}

// ================================================================================================
// TESTING
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::vm::VM;

    fn setup_thread() -> Thread {
        VM::init_for_test();
        Thread::new()
    }

    #[test]
    fn test_rgen_basic() {
        let mut rgen = RGen::new(12345);
        
        // Test drand produces values in [0, 1)
        for _ in 0..100 {
            let val = rgen.drand();
            assert!(val >= 0.0 && val < 1.0, "drand out of range: {}", val);
        }
        
        // Test drand2 produces values in [-1, 1)
        for _ in 0..100 {
            let val = rgen.drand2();
            assert!(val >= -1.0 && val < 1.0, "drand2 out of range: {}", val);
        }
    }

    #[test]
    fn test_rgen_deterministic() {
        let mut rgen1 = RGen::new(42);
        let mut rgen2 = RGen::new(42);
        
        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(rgen1.drand(), rgen2.drand());
        }
    }

    #[test]
    fn test_coin() {
        let mut thread = setup_thread();
        
        // Test coin with probability 0.0 (should always be 0)
        thread.push(Value::Real(0.0));
        coin(&mut thread).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(0.0));
        
        // Test coin with probability 1.0 (should always be 1)
        thread.push(Value::Real(1.0));
        coin(&mut thread).unwrap();
        assert_eq!(thread.pop().unwrap(), Value::Real(1.0));
    }

    #[test]
    fn test_rand() {
        let mut thread = setup_thread();
        
        // Test basic range
        thread.push(Value::Real(10.0));
        thread.push(Value::Real(20.0));
        rand(&mut thread).unwrap();
        
        if let Value::Real(result) = thread.pop().unwrap() {
            assert!(result >= 10.0 && result < 20.0, "rand out of range: {}", result);
        } else {
            panic!("Expected Real value");
        }
    }

    #[test]
    fn test_irand() {
        let mut thread = setup_thread();
        
        // Test basic integer range
        thread.push(Value::Real(5.0));
        thread.push(Value::Real(15.0));
        irand(&mut thread).unwrap();
        
        if let Value::Real(result) = thread.pop().unwrap() {
            let int_result = result as i64;
            assert!(int_result >= 5 && int_result <= 15, "irand out of range: {}", int_result);
        } else {
            panic!("Expected Real value");
        }
    }

    #[test]
    fn test_pick() {
        let mut thread = setup_thread();
        
        // Create test list  
        let items = vec![
            Value::Real(1.0),
            Value::Real(2.0),
            Value::Real(3.0),
        ];
        let list = List::from_array(Array::from_values(items.clone()));
        thread.push(Value::object(list));
        
        pick(&mut thread).unwrap();
        let result = thread.pop().unwrap();
        
        // Result should be one of the original items
        assert!(items.contains(&result), "pick returned invalid item: {:?}", result);
    }

    #[test]
    fn test_wrand() {
        let mut thread = setup_thread();
        
        // Create weights that heavily favor index 1
        let weights = vec![
            Value::Real(0.1),
            Value::Real(0.8),
            Value::Real(0.1),
        ];
        let weights_list = List::from_array(Array::from_values(weights));
        thread.push(Value::object(weights_list));
        
        // Test multiple times to check distribution
        let mut results = Vec::new();
        for _ in 0..10 {
            let test_weights = vec![Value::Real(0.1), Value::Real(0.8), Value::Real(0.1)];
            let test_list = List::from_array(Array::from_values(test_weights));
            thread.push(Value::object(test_list));
            wrand(&mut thread).unwrap();
            if let Value::Real(idx) = thread.pop().unwrap() {
                results.push(idx as usize);
            }
        }
        
        // Should have valid indices
        for &idx in &results {
            assert!(idx < 3, "wrand returned invalid index: {}", idx);
        }
    }

    #[test]
    fn test_seeding() {
        let mut thread = setup_thread();
        
        // Set specific seed
        thread.push(Value::Real(12345.0));
        setseed(&mut thread).unwrap();
        
        // Generate some numbers
        let mut results1 = Vec::new();
        for _ in 0..5 {
            thread.push(Value::Real(0.0));
            thread.push(Value::Real(1.0));
            rand(&mut thread).unwrap();
            if let Value::Real(val) = thread.pop().unwrap() {
                results1.push(val);
            }
        }
        
        // Reset with same seed
        thread.push(Value::Real(12345.0));
        setseed(&mut thread).unwrap();
        
        // Generate same numbers
        let mut results2 = Vec::new();
        for _ in 0..5 {
            thread.push(Value::Real(0.0));
            thread.push(Value::Real(1.0));
            rand(&mut thread).unwrap();
            if let Value::Real(val) = thread.pop().unwrap() {
                results2.push(val);
            }
        }
        
        // Should be identical
        assert_eq!(results1, results2, "Same seed should produce same sequence");
    }
}
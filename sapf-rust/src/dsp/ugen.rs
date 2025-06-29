//! Unit Generator (UGen) base types and traits
//!
//! This module provides the foundation for all audio generators in SAPF.
//! It includes the base UGen trait and common generator patterns.

use crate::core::{SapfError, Result, Value, Object};
use crate::vm::Thread;
use std::sync::Arc;
use std::fmt;

/// Audio sample type - matches the C++ implementation's Z type
pub type Sample = f64;

/// Block size for audio processing
/// This should match the VM's audio block size settings
pub const DEFAULT_BLOCK_SIZE: usize = 64;

/// Base trait for all Unit Generators
/// 
/// A UGen is an audio generator that produces a stream of audio samples.
/// This corresponds to the Gen class in the C++ implementation.
pub trait UGen: Object {
    /// Pull audio samples from this generator
    /// 
    /// This is the core method that generates audio. It should fill the
    /// output buffer with the requested number of samples.
    fn pull(&mut self, thread: &mut Thread, output: &mut [Sample]) -> Result<usize>;
    
    /// Check if this generator is finished
    fn is_done(&self) -> bool;
    
    /// Mark this generator as finished
    fn set_done(&mut self);
    
    /// Get the block size for this generator
    fn block_size(&self) -> usize {
        DEFAULT_BLOCK_SIZE
    }
    
    /// Get the type name for debugging
    fn ugen_type_name(&self) -> &'static str;
}

/// Base implementation for UGens that provides common functionality
#[derive(Debug)]
pub struct UGenBase {
    done: bool,
    block_size: usize,
    finite: bool,
}

impl UGenBase {
    /// Create a new UGen base with the specified properties
    pub fn new(block_size: usize, finite: bool) -> Self {
        Self {
            done: false,
            block_size,
            finite,
        }
    }
    
    /// Check if this UGen is finite (will eventually finish)
    pub fn is_finite(&self) -> bool {
        self.finite
    }
    
    /// Get the block size
    pub fn block_size(&self) -> usize {
        self.block_size
    }
    
    /// Check if done
    pub fn is_done(&self) -> bool {
        self.done
    }
    
    /// Set done flag
    pub fn set_done(&mut self) {
        self.done = true;
    }
}

/// Trait for zero-input generators (sources)
/// 
/// These are generators that produce audio without any audio inputs,
/// such as oscillators, noise generators, etc.
pub trait ZeroInputGen: UGen {
    /// Calculate audio samples
    /// 
    /// # Arguments
    /// * `samples` - Number of samples to generate
    /// * `output` - Output buffer to fill
    fn calc(&mut self, samples: usize, output: &mut [Sample]);
}

/// Trait for single-input generators
/// 
/// These are generators that process one audio input, such as
/// filters, amplifiers, etc.
pub trait OneInputGen: UGen {
    /// Calculate audio samples with one input
    /// 
    /// # Arguments
    /// * `samples` - Number of samples to process
    /// * `output` - Output buffer to fill
    /// * `input` - Input audio samples
    fn calc(&mut self, samples: usize, output: &mut [Sample], input: &[Sample]);
}

/// Trait for two-input generators
/// 
/// These are generators that process two audio inputs, such as
/// mixers, modulators, etc.
pub trait TwoInputGen: UGen {
    /// Calculate audio samples with two inputs
    /// 
    /// # Arguments
    /// * `samples` - Number of samples to process
    /// * `output` - Output buffer to fill
    /// * `input_a` - First input audio samples
    /// * `input_b` - Second input audio samples
    fn calc(&mut self, samples: usize, output: &mut [Sample], 
            input_a: &[Sample], input_b: &[Sample]);
}

/// A simple constant generator for testing
#[derive(Debug)]
pub struct ConstantGen {
    base: UGenBase,
    value: Sample,
}

impl ConstantGen {
    /// Create a new constant generator
    pub fn new(value: Sample) -> Self {
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false), // Infinite generator
            value,
        }
    }
}

impl Object for ConstantGen {
    fn as_float(&self) -> Result<f64> {
        Ok(self.value)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.value))
    }
    
    fn type_name(&self) -> &'static str {
        "ConstantGen"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(ConstantGen::new(self.value))
    }
}

impl fmt::Display for ConstantGen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ConstantGen({})", self.value)
    }
}

impl UGen for ConstantGen {
    fn pull(&mut self, _thread: &mut Thread, output: &mut [Sample]) -> Result<usize> {
        let samples_to_fill = output.len().min(self.base.block_size());
        
        // Fill output with constant value
        for i in 0..samples_to_fill {
            output[i] = self.value;
        }
        
        Ok(samples_to_fill)
    }
    
    fn is_done(&self) -> bool {
        self.base.is_done()
    }
    
    fn set_done(&mut self) {
        self.base.set_done();
    }
    
    fn block_size(&self) -> usize {
        self.base.block_size()
    }
    
    fn ugen_type_name(&self) -> &'static str {
        "ConstantGen"
    }
}

impl ZeroInputGen for ConstantGen {
    fn calc(&mut self, samples: usize, output: &mut [Sample]) {
        for i in 0..samples {
            output[i] = self.value;
        }
    }
}

/// A simple sine wave oscillator
#[derive(Debug)]
pub struct SineOsc {
    base: UGenBase,
    frequency: Sample,
    phase: Sample,
    sample_rate: Sample,
}

impl SineOsc {
    /// Create a new sine oscillator
    pub fn new(frequency: Sample, sample_rate: Sample) -> Self {
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false), // Infinite generator
            frequency,
            phase: 0.0,
            sample_rate,
        }
    }
    
    /// Set the frequency
    pub fn set_frequency(&mut self, frequency: Sample) {
        self.frequency = frequency;
    }
}

impl Object for SineOsc {
    fn as_float(&self) -> Result<f64> {
        Ok(self.frequency)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.frequency))
    }
    
    fn type_name(&self) -> &'static str {
        "SineOsc"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(SineOsc::new(self.frequency, self.sample_rate))
    }
}

impl fmt::Display for SineOsc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SineOsc(freq={}Hz)", self.frequency)
    }
}

impl UGen for SineOsc {
    fn pull(&mut self, _thread: &mut Thread, output: &mut [Sample]) -> Result<usize> {
        let samples_to_fill = output.len().min(self.base.block_size());
        
        self.calc(samples_to_fill, &mut output[..samples_to_fill]);
        
        Ok(samples_to_fill)
    }
    
    fn is_done(&self) -> bool {
        self.base.is_done()
    }
    
    fn set_done(&mut self) {
        self.base.set_done();
    }
    
    fn block_size(&self) -> usize {
        self.base.block_size()
    }
    
    fn ugen_type_name(&self) -> &'static str {
        "SineOsc"
    }
}

impl ZeroInputGen for SineOsc {
    fn calc(&mut self, samples: usize, output: &mut [Sample]) {
        let phase_increment = 2.0 * std::f64::consts::PI * self.frequency / self.sample_rate;
        
        for i in 0..samples {
            output[i] = self.phase.sin();
            self.phase += phase_increment;
            
            // Keep phase in range [0, 2π]
            if self.phase >= 2.0 * std::f64::consts::PI {
                self.phase -= 2.0 * std::f64::consts::PI;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{Thread, VM};

    #[test]
    fn test_constant_generator() {
        let _vm = VM::instance(); // Initialize VM for test
        let mut thread = Thread::new();
        let mut gen = ConstantGen::new(440.0);
        let mut output = [0.0; 10];
        
        let samples = gen.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 10);
        
        // All samples should be 440.0
        for &sample in &output {
            assert_eq!(sample, 440.0);
        }
        
        assert!(!gen.is_done());
        assert_eq!(gen.ugen_type_name(), "ConstantGen");
    }

    #[test]
    fn test_sine_oscillator() {
        let _vm = VM::instance(); // Initialize VM for test
        let mut thread = Thread::new();
        let mut osc = SineOsc::new(1.0, 44100.0); // 1 Hz at 44.1 kHz
        let mut output = [0.0; 4];
        
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 4);
        
        // First sample should be sin(0) = 0
        assert!((output[0] - 0.0).abs() < 1e-10);
        
        // Oscillator should not be done
        assert!(!osc.is_done());
        assert_eq!(osc.ugen_type_name(), "SineOsc");
    }

    #[test]
    fn test_sine_oscillator_frequency() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = SineOsc::new(440.0, 44100.0);
        
        // Test frequency setting
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency, 880.0);
        
        let mut output = [0.0; 1];
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 1);
    }

    #[test]
    fn test_ugen_base() {
        let mut base = UGenBase::new(128, true);
        
        assert_eq!(base.block_size(), 128);
        assert!(base.is_finite());
        assert!(!base.is_done());
        
        base.set_done();
        assert!(base.is_done());
    }

    #[test]
    fn test_ugen_as_object() {
        let gen = ConstantGen::new(42.0);
        
        // Test Object trait methods
        assert_eq!(gen.as_float().unwrap(), 42.0);
        assert_eq!(gen.type_name(), "ConstantGen");
        
        if let Value::Real(val) = gen.deref().unwrap() {
            assert_eq!(val, 42.0);
        } else {
            panic!("Expected Real value");
        }
    }

    #[test]
    fn test_block_size_limits() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut gen = ConstantGen::new(1.0);
        let mut large_output = [0.0; 1000];
        
        // Should only fill up to block size
        let samples = gen.pull(&mut thread, &mut large_output).unwrap();
        assert_eq!(samples, DEFAULT_BLOCK_SIZE);
        
        // Check that only the first block_size samples are filled
        for i in 0..DEFAULT_BLOCK_SIZE {
            assert_eq!(large_output[i], 1.0);
        }
    }
}
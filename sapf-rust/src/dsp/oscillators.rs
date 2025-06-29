//! Basic Oscillators for SAPF
//!
//! This module implements fundamental audio oscillators including sine, sawtooth,
//! and pulse waves. These correspond to the basic oscillators in OscilUGens.cpp.

use crate::core::{SapfError, Result, Value, Object, math};
use crate::vm::{Thread, VM};
use crate::dsp::ugen::{UGen, UGenBase, ZeroInputGen, OneInputGen, Sample, DEFAULT_BLOCK_SIZE};
use std::sync::Arc;
use std::fmt;

/// Two pi constant for phase calculations
const TWO_PI: Sample = 2.0 * std::f64::consts::PI;

/// Phase wrapping helper - ensures phase stays in [0, 2π) range
fn wrap_phase(phase: Sample) -> Sample {
    if phase >= TWO_PI {
        phase - TWO_PI
    } else if phase < 0.0 {
        phase + TWO_PI
    } else {
        phase
    }
}

/// Low-frequency sawtooth oscillator
/// 
/// Generates a sawtooth wave that ramps from -1 to 1.
/// This corresponds to LFSaw in the C++ implementation.
#[derive(Debug)]
pub struct SawtoothOsc {
    base: UGenBase,
    phase: Sample,        // Current phase in range [-1, 1]
    frequency: Sample,
    phase_increment: Sample,
    sample_rate: Sample,
}

impl SawtoothOsc {
    /// Create a new sawtooth oscillator
    pub fn new(frequency: Sample, initial_phase: Sample, sample_rate: Sample) -> Self {
        let wrapped_phase = if initial_phase >= 1.0 { 
            initial_phase - 2.0 
        } else if initial_phase < -1.0 { 
            initial_phase + 2.0 
        } else { 
            initial_phase * 2.0 - 1.0  // Convert [0,1] to [-1,1]
        };
        
        let phase_increment = 2.0 * frequency / sample_rate; // 2.0 for [-1,1] range
        
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false),
            phase: wrapped_phase,
            frequency,
            phase_increment,
            sample_rate,
        }
    }
    
    /// Set the frequency
    pub fn set_frequency(&mut self, frequency: Sample) {
        self.frequency = frequency;
        self.phase_increment = 2.0 * frequency / self.sample_rate;
    }
}

impl Object for SawtoothOsc {
    fn as_float(&self) -> Result<f64> {
        Ok(self.frequency)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.frequency))
    }
    
    fn type_name(&self) -> &'static str {
        "SawtoothOsc"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(SawtoothOsc::new(self.frequency, self.phase, self.sample_rate))
    }
}

impl fmt::Display for SawtoothOsc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SawtoothOsc(freq={}Hz)", self.frequency)
    }
}

impl UGen for SawtoothOsc {
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
        "SawtoothOsc"
    }
}

impl ZeroInputGen for SawtoothOsc {
    fn calc(&mut self, samples: usize, output: &mut [Sample]) {
        for i in 0..samples {
            output[i] = self.phase;
            self.phase += self.phase_increment;
            
            // Wrap phase to [-1, 1]
            if self.phase >= 1.0 {
                self.phase -= 2.0;
            } else if self.phase < -1.0 {
                self.phase += 2.0;
            }
        }
    }
}

/// Low-frequency pulse oscillator
/// 
/// Generates a pulse wave with variable duty cycle.
/// This corresponds to LFPulse in the C++ implementation.
#[derive(Debug)]
pub struct PulseOsc {
    base: UGenBase,
    phase: Sample,        // Current phase in range [0, 1]
    frequency: Sample,
    duty_cycle: Sample,   // Duty cycle in range [0, 1]
    phase_increment: Sample,
    sample_rate: Sample,
    bipolar: bool,        // If true, output [-1, 1], else [0, 1]
}

impl PulseOsc {
    /// Create a new pulse oscillator
    pub fn new(frequency: Sample, initial_phase: Sample, duty_cycle: Sample, sample_rate: Sample, bipolar: bool) -> Self {
        let wrapped_phase = initial_phase - initial_phase.floor(); // Wrap to [0, 1]
        let clamped_duty = duty_cycle.clamp(0.0, 1.0);
        let phase_increment = frequency / sample_rate;
        
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false),
            phase: wrapped_phase,
            frequency,
            duty_cycle: clamped_duty,
            phase_increment,
            sample_rate,
            bipolar,
        }
    }
    
    /// Create a bipolar pulse oscillator (output range [-1, 1])
    pub fn new_bipolar(frequency: Sample, initial_phase: Sample, duty_cycle: Sample, sample_rate: Sample) -> Self {
        Self::new(frequency, initial_phase, duty_cycle, sample_rate, true)
    }
    
    /// Create a unipolar pulse oscillator (output range [0, 1])  
    pub fn new_unipolar(frequency: Sample, initial_phase: Sample, duty_cycle: Sample, sample_rate: Sample) -> Self {
        Self::new(frequency, initial_phase, duty_cycle, sample_rate, false)
    }
    
    /// Set the frequency
    pub fn set_frequency(&mut self, frequency: Sample) {
        self.frequency = frequency;
        self.phase_increment = frequency / self.sample_rate;
    }
    
    /// Set the duty cycle
    pub fn set_duty_cycle(&mut self, duty_cycle: Sample) {
        self.duty_cycle = duty_cycle.clamp(0.0, 1.0);
    }
}

impl Object for PulseOsc {
    fn as_float(&self) -> Result<f64> {
        Ok(self.frequency)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.frequency))
    }
    
    fn type_name(&self) -> &'static str {
        if self.bipolar { "PulseOscBipolar" } else { "PulseOsc" }
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(PulseOsc::new(self.frequency, self.phase, self.duty_cycle, self.sample_rate, self.bipolar))
    }
}

impl fmt::Display for PulseOsc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PulseOsc(freq={}Hz, duty={})", self.frequency, self.duty_cycle)
    }
}

impl UGen for PulseOsc {
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
        if self.bipolar { "PulseOscBipolar" } else { "PulseOsc" }
    }
}

impl ZeroInputGen for PulseOsc {
    fn calc(&mut self, samples: usize, output: &mut [Sample]) {
        for i in 0..samples {
            let pulse_value = if self.phase < self.duty_cycle { 1.0 } else { 0.0 };
            
            output[i] = if self.bipolar {
                pulse_value * 2.0 - 1.0  // Convert [0,1] to [-1,1]
            } else {
                pulse_value
            };
            
            self.phase += self.phase_increment;
            
            // Wrap phase to [0, 1]
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }
}

/// Enhanced sine oscillator using lookup tables
/// 
/// This version uses the optimized sine lookup tables from math.rs
/// for better performance, similar to the C++ implementation.
#[derive(Debug)]
pub struct SineOscLUT {
    base: UGenBase,
    phase: Sample,        // Current phase in radians
    frequency: Sample,
    phase_increment: Sample,
    sample_rate: Sample,
}

impl SineOscLUT {
    /// Create a new sine oscillator with lookup table optimization
    pub fn new(frequency: Sample, initial_phase: Sample, sample_rate: Sample) -> Self {
        let wrapped_phase = wrap_phase(initial_phase * TWO_PI);
        let phase_increment = TWO_PI * frequency / sample_rate;
        
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false),
            phase: wrapped_phase,
            frequency,
            phase_increment,
            sample_rate,
        }
    }
    
    /// Set the frequency
    pub fn set_frequency(&mut self, frequency: Sample) {
        self.frequency = frequency;
        self.phase_increment = TWO_PI * frequency / self.sample_rate;
    }
}

impl Object for SineOscLUT {
    fn as_float(&self) -> Result<f64> {
        Ok(self.frequency)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.frequency))
    }
    
    fn type_name(&self) -> &'static str {
        "SineOscLUT"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(SineOscLUT::new(self.frequency, self.phase / TWO_PI, self.sample_rate))
    }
}

impl fmt::Display for SineOscLUT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SineOscLUT(freq={}Hz)", self.frequency)
    }
}

impl UGen for SineOscLUT {
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
        "SineOscLUT"
    }
}

impl ZeroInputGen for SineOscLUT {
    fn calc(&mut self, samples: usize, output: &mut [Sample]) {
        for i in 0..samples {
            // Use optimized sine lookup from math.rs
            output[i] = math::tsin(self.phase);
            
            self.phase += self.phase_increment;
            self.phase = wrap_phase(self.phase);
        }
    }
}

/// Variable frequency sine oscillator
/// 
/// This oscillator accepts frequency modulation input, similar to SinOsc
/// in the C++ implementation.
#[derive(Debug)]
pub struct SineOscFM {
    base: UGenBase,
    phase: Sample,
    base_frequency: Sample,
    phase_multiplier: Sample,  // radians per sample at base frequency
    sample_rate: Sample,
}

impl SineOscFM {
    /// Create a new frequency-modulated sine oscillator
    pub fn new(base_frequency: Sample, initial_phase: Sample, sample_rate: Sample) -> Self {
        let wrapped_phase = wrap_phase(initial_phase * TWO_PI);
        let phase_multiplier = TWO_PI / sample_rate;
        
        Self {
            base: UGenBase::new(DEFAULT_BLOCK_SIZE, false),
            phase: wrapped_phase,
            base_frequency,
            phase_multiplier,
            sample_rate,
        }
    }
    
    /// Set the base frequency (used when no modulation input)
    pub fn set_base_frequency(&mut self, frequency: Sample) {
        self.base_frequency = frequency;
    }
}

impl Object for SineOscFM {
    fn as_float(&self) -> Result<f64> {
        Ok(self.base_frequency)
    }
    
    fn deref(&self) -> Result<Value> {
        Ok(Value::Real(self.base_frequency))
    }
    
    fn type_name(&self) -> &'static str {
        "SineOscFM"
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn clone_object(&self) -> Arc<dyn Object> {
        Arc::new(SineOscFM::new(self.base_frequency, self.phase / TWO_PI, self.sample_rate))
    }
}

impl fmt::Display for SineOscFM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SineOscFM(base_freq={}Hz)", self.base_frequency)
    }
}

impl UGen for SineOscFM {
    fn pull(&mut self, _thread: &mut Thread, output: &mut [Sample]) -> Result<usize> {
        let samples_to_fill = output.len().min(self.base.block_size());
        
        // For now, just use base frequency (no modulation input)
        for i in 0..samples_to_fill {
            output[i] = self.phase.sin();
            self.phase += self.base_frequency * self.phase_multiplier;
            self.phase = wrap_phase(self.phase);
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
        "SineOscFM"
    }
}

impl OneInputGen for SineOscFM {
    fn calc(&mut self, samples: usize, output: &mut [Sample], freq_input: &[Sample]) {
        for i in 0..samples {
            // Use lookup table for sine calculation
            output[i] = math::tsin(self.phase);
            
            // Use frequency from input if available, otherwise base frequency
            let frequency = if i < freq_input.len() { 
                freq_input[i] 
            } else { 
                self.base_frequency 
            };
            
            self.phase += frequency * self.phase_multiplier;
            self.phase = wrap_phase(self.phase);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::{Thread, VM};

    #[test]
    fn test_sawtooth_oscillator() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = SawtoothOsc::new(1.0, 0.0, 8.0); // 1 Hz at 8 samples/sec
        let mut output = [0.0; 8];
        
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 8);
        
        // At 1Hz and 8 samples/sec, we should get a complete cycle
        // Phase should ramp from -1 to just under 1
        assert!(output[0] >= -1.0 && output[0] < -0.5); // Start near -1
        assert!(output[7] >= 0.5 && output[7] < 1.0);   // End near 1
        
        // Should be monotonically increasing
        for i in 1..8 {
            assert!(output[i] > output[i-1], "Sawtooth should be increasing");
        }
    }

    #[test]
    fn test_pulse_oscillator_unipolar() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = PulseOsc::new_unipolar(1.0, 0.0, 0.5, 4.0); // 1 Hz, 50% duty, 4 samples/sec
        let mut output = [0.0; 4];
        
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 4);
        
        // With 50% duty cycle and 4 samples/sec, first 2 should be high, last 2 low
        assert_eq!(output[0], 1.0);
        assert_eq!(output[1], 1.0);
        assert_eq!(output[2], 0.0);
        assert_eq!(output[3], 0.0);
    }

    #[test]
    fn test_pulse_oscillator_bipolar() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = PulseOsc::new_bipolar(2.0, 0.0, 0.25, 8.0); // 2 Hz, 25% duty, 8 samples/sec
        let mut output = [0.0; 4];
        
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 4);
        
        // At 2Hz with 8 samples/sec, we get 4 samples per cycle
        // 25% duty means first sample high, rest low
        assert_eq!(output[0], 1.0);  // High
        assert_eq!(output[1], -1.0); // Low
        assert_eq!(output[2], -1.0); // Low  
        assert_eq!(output[3], -1.0); // Low
    }

    #[test]
    fn test_sine_oscillator_lut() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = SineOscLUT::new(1.0, 0.0, 44100.0);
        let mut output = [0.0; 4];
        
        let samples = osc.pull(&mut thread, &mut output).unwrap();
        assert_eq!(samples, 4);
        
        // First sample should be sin(0) ≈ 0
        assert!((output[0] - 0.0).abs() < 1e-10);
        
        // Values should be in [-1, 1] range
        for &sample in &output {
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_sine_oscillator_fm() {
        let _vm = VM::instance();
        let mut thread = Thread::new();
        let mut osc = SineOscFM::new(440.0, 0.0, 44100.0);
        
        // Test with frequency modulation
        let freq_input = [440.0, 880.0, 1320.0, 440.0];
        let mut output = [0.0; 4];
        
        osc.calc(4, &mut output, &freq_input);
        
        // Output should be in valid range
        for &sample in &output {
            assert!(sample >= -1.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_oscillator_frequency_setting() {
        let _vm = VM::instance();
        let mut saw = SawtoothOsc::new(440.0, 0.0, 44100.0);
        let mut pulse = PulseOsc::new_unipolar(440.0, 0.0, 0.5, 44100.0);
        let mut sine = SineOscLUT::new(440.0, 0.0, 44100.0);
        let mut sine_fm = SineOscFM::new(440.0, 0.0, 44100.0);
        
        // Test frequency setting
        saw.set_frequency(880.0);
        pulse.set_frequency(880.0);
        sine.set_frequency(880.0);
        sine_fm.set_base_frequency(880.0);
        
        assert_eq!(saw.frequency, 880.0);
        assert_eq!(pulse.frequency, 880.0);
        assert_eq!(sine.frequency, 880.0);
        assert_eq!(sine_fm.base_frequency, 880.0);
    }

    #[test]
    fn test_pulse_duty_cycle() {
        let mut osc = PulseOsc::new_unipolar(1.0, 0.0, 0.7, 44100.0);
        
        osc.set_duty_cycle(0.3);
        assert_eq!(osc.duty_cycle, 0.3);
        
        // Test clamping
        osc.set_duty_cycle(1.5);
        assert_eq!(osc.duty_cycle, 1.0);
        
        osc.set_duty_cycle(-0.1);
        assert_eq!(osc.duty_cycle, 0.0);
    }

    #[test]
    fn test_oscillator_as_objects() {
        let saw = SawtoothOsc::new(440.0, 0.0, 44100.0);
        let pulse = PulseOsc::new_unipolar(440.0, 0.0, 0.5, 44100.0);
        let sine = SineOscLUT::new(440.0, 0.0, 44100.0);
        
        // Test Object trait methods
        assert_eq!(saw.as_float().unwrap(), 440.0);
        assert_eq!(pulse.as_float().unwrap(), 440.0);
        assert_eq!(sine.as_float().unwrap(), 440.0);
        
        assert_eq!(saw.type_name(), "SawtoothOsc");
        assert_eq!(pulse.type_name(), "PulseOsc");
        assert_eq!(sine.type_name(), "SineOscLUT");
    }

    #[test]
    fn test_phase_wrapping() {
        assert!((wrap_phase(0.0) - 0.0).abs() < 1e-10);
        assert!((wrap_phase(TWO_PI) - 0.0).abs() < 1e-10);
        assert!((wrap_phase(-0.1) - (TWO_PI - 0.1)).abs() < 1e-10);
        assert!((wrap_phase(TWO_PI + 0.1) - 0.1).abs() < 1e-10);
    }
}
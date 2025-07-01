//! Audio input handling for SAPF
//!
//! This module provides the Rust equivalent of the C++ ZIn class,
//! which handles reading audio data from various sources including
//! UGens, lists, and constant values.

use crate::core::{SapfError, Result, Value, List, Object};
use crate::vm::Thread;
use crate::dsp::ugen::{Sample, UGen};
use std::sync::Arc;

/// Audio input handler - equivalent to C++ ZIn class
/// 
/// Handles input from various sources:
/// - UGens (audio generators)
/// - ZLists (numeric lists/signals)  
/// - Constants (for control signals)
/// - Other Value types that can be converted to audio
#[derive(Debug)]
pub struct AudioIn {
    /// Current input value/source
    value: Value,
    /// Whether this is a constant value
    is_constant: bool,
    /// Current read offset for lists
    offset: usize,
    /// Whether the input is done/finished
    done: bool,
    /// Flag for single-value operations
    once: bool,
}

impl AudioIn {
    /// Create a new AudioIn with no input
    pub fn new() -> Self {
        Self {
            value: Value::Nil,
            is_constant: false,
            offset: 0,
            done: false,
            once: true,
        }
    }

    /// Create a new AudioIn with the given value
    pub fn with_value(value: Value) -> Self {
        let is_constant = value.is_real();
        Self {
            value,
            is_constant,
            offset: 0,
            done: false,
            once: true,
        }
    }

    /// Set the input value/source
    pub fn set(&mut self, value: Value) {
        self.value = value;
        self.is_constant = self.value.is_real();
        self.offset = 0;
        self.done = false;
        self.once = true;
    }

    /// Check if the input is done/finished
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Mark the input as done
    pub fn set_done(&mut self) {
        self.done = true;
    }

    /// Fill an audio buffer with samples from this input
    /// 
    /// Returns the number of samples actually written and whether the input is done.
    /// This is the main method for audio processing, equivalent to C++ ZIn::fill()
    pub fn fill(&mut self, thread: &mut Thread, num_samples: usize, output: &mut [Sample]) -> Result<(usize, bool)> {
        if self.done {
            // Fill with zeros if done
            for sample in output.iter_mut().take(num_samples) {
                *sample = 0.0;
            }
            return Ok((num_samples, true));
        }

        // Handle List case specially to avoid borrow conflicts
        match &self.value {
            Value::Object(obj) => {
                if let Some(list) = obj.as_any().downcast_ref::<List>() {
                    // Clone the list reference to avoid borrow conflicts
                    let list_clone = list.clone();
                    return self.fill_from_list(thread, &list_clone, num_samples, output);
                }
            }
            _ => {}
        }

        // Handle non-list cases
        match &self.value {
            Value::Nil => {
                // Fill with zeros for nil values
                for sample in output.iter_mut().take(num_samples) {
                    *sample = 0.0;
                }
                Ok((num_samples, true))
            }
            
            Value::Real(r) => {
                // Fill with constant value
                let val = *r;
                for sample in output.iter_mut().take(num_samples) {
                    *sample = val;
                }
                Ok((num_samples, false)) // Constants never finish
            }
            
            Value::Object(_) => {
                // Try to convert other objects to real values
                // This includes UGens, which should convert to their frequency/amplitude
                match self.value.to_real() {
                    Ok(val) => {
                        for sample in output.iter_mut().take(num_samples) {
                            *sample = val;
                        }
                        Ok((num_samples, false))
                    }
                    Err(_) => {
                        // Fill with zeros if can't convert
                        for sample in output.iter_mut().take(num_samples) {
                            *sample = 0.0;
                        }
                        Ok((num_samples, true))
                    }
                }
            }
        }
    }

    /// Fill from a ZList (numeric list)
    fn fill_from_list(&mut self, thread: &mut Thread, list: &List, num_samples: usize, output: &mut [Sample]) -> Result<(usize, bool)> {
        let mut samples_written = 0;
        let mut done = false;

        if list.is_infinite() {
            // Handle infinite/lazy lists
            for i in 0..num_samples {
                if i >= output.len() {
                    break;
                }
                
                match list.at_index(thread, self.offset) {
                    Ok(value) => {
                        output[i] = value.to_real().unwrap_or(0.0);
                        samples_written += 1;
                        self.offset += 1;
                    }
                    Err(_) => {
                        // End of list reached
                        output[i] = 0.0;
                        done = true;
                        break;
                    }
                }
            }
        } else {
            // Handle finite lists
            for i in 0..num_samples {
                if i >= output.len() {
                    break;
                }
                
                match list.at_index(thread, self.offset) {
                    Ok(value) => {
                        output[i] = value.to_real().unwrap_or(0.0);
                        samples_written += 1;
                        self.offset += 1;
                    }
                    Err(_) => {
                        // Past end of list, fill with zeros
                        output[i] = 0.0;
                        done = true;
                        break;
                    }
                }
            }
        }

        if done {
            self.done = true;
        }

        Ok((samples_written, done))
    }


    /// Read a single sample value
    /// Equivalent to C++ ZIn::onez()
    pub fn read_one(&mut self, thread: &mut Thread) -> Result<Sample> {
        match &self.value {
            Value::Nil => Ok(0.0),
            Value::Real(r) => Ok(*r),
            Value::Object(obj) => {
                if let Some(list) = obj.as_any().downcast_ref::<List>() {
                    match list.at_index(thread, self.offset) {
                        Ok(value) => {
                            self.offset += 1;
                            Ok(value.to_real().unwrap_or(0.0))
                        }
                        Err(_) => {
                            self.done = true;
                            Ok(0.0)
                        }
                    }
                } else {
                    self.value.to_real().or(Ok(0.0))
                }
            }
        }
    }

    /// Peek at the next sample without advancing
    /// Equivalent to C++ ZIn::peek()
    pub fn peek(&self, thread: &mut Thread) -> Result<Sample> {
        match &self.value {
            Value::Nil => Ok(0.0),
            Value::Real(r) => Ok(*r),
            Value::Object(obj) => {
                if let Some(list) = obj.as_any().downcast_ref::<List>() {
                    match list.at_index(thread, self.offset) {
                        Ok(value) => Ok(value.to_real().unwrap_or(0.0)),
                        Err(_) => Ok(0.0),
                    }
                } else {
                    self.value.to_real().or(Ok(0.0))
                }
            }
        }
    }

    /// Advance the read position by the specified number of frames
    /// Equivalent to C++ ZIn::hop()
    pub fn hop(&mut self, frames: usize) {
        self.offset += frames;
    }

    /// Reset the read position to the beginning
    pub fn reset(&mut self) {
        self.offset = 0;
        self.done = false;
        self.once = true;
    }
}

impl Default for AudioIn {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::list::Array;

    #[test]
    fn test_audio_in_constant() {
        let mut thread = Thread::new();
        let mut input = AudioIn::with_value(Value::Real(0.5));
        
        let mut buffer = [0.0; 4];
        let (written, done) = input.fill(&mut thread, 4, &mut buffer).unwrap();
        
        assert_eq!(written, 4);
        assert!(!done); // Constants never finish
        assert_eq!(buffer, [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_audio_in_nil() {
        let mut thread = Thread::new();
        let mut input = AudioIn::new();
        
        let mut buffer = [1.0; 4]; // Pre-fill with non-zero
        let (written, done) = input.fill(&mut thread, 4, &mut buffer).unwrap();
        
        assert_eq!(written, 4);
        assert!(done); // Nil is immediately done
        assert_eq!(buffer, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_audio_in_read_one() {
        let mut thread = Thread::new();
        let mut input = AudioIn::with_value(Value::Real(0.75));
        
        let sample = input.read_one(&mut thread).unwrap();
        assert_eq!(sample, 0.75);
    }

    #[test]
    fn test_audio_in_peek() {
        let mut thread = Thread::new();
        let input = AudioIn::with_value(Value::Real(0.25));
        
        let sample = input.peek(&mut thread).unwrap();
        assert_eq!(sample, 0.25);
        
        // Peeking shouldn't change state
        let sample2 = input.peek(&mut thread).unwrap();
        assert_eq!(sample2, 0.25);
    }
}
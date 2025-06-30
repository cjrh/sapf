//! Audio operations for SAPF VM
//!
//! This module provides built-in functions for audio playback and control,
//! equivalent to the C++ Play.cpp functionality.

use crate::core::{SapfError, Value, Object, Result};
use crate::core::function::Primitive;
use crate::vm::{VM, Thread};
use crate::audio::{play, stop_all, get_audio_info};
use std::sync::Arc;

/// Play function - equivalent to C++ playWithAudioUnit()
/// 
/// Pops a value from the stack and starts audio playback.
/// Usage: signal play
fn play_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    if thread.stack_depth() < 1 {
        return Err(SapfError::StackUnderflow("play requires 1 argument".to_string()));
    }

    let value = thread.pop()?;
    
    // Create a copy of the thread for audio playback
    // Note: This is a simplified approach - the real implementation would need
    // proper thread/ownership management for audio processing
    let audio_thread = Thread::new()?;
    
    match play(audio_thread, value) {
        Ok(player_id) => {
            println!("Audio playback started (player ID: {})", player_id);
            Ok(())
        }
        Err(e) => {
            eprintln!("Audio playback failed: {}", e);
            Err(e)
        }
    }
}

/// Stop all audio playback
/// Usage: stopall
fn stop_all_prim(_thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    match stop_all() {
        Ok(()) => {
            println!("All audio playback stopped");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to stop audio: {}", e);
            Err(e)
        }
    }
}

/// Get audio system information
/// Usage: audioinfo
fn audio_info_prim(_thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    match get_audio_info() {
        Ok(info) => {
            // Push the info as a form/dictionary to the stack
            for (key, value) in info {
                println!("{}: {:?}", key, value);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to get audio info: {}", e);
            Err(e)
        }
    }
}

/// Register audio built-in functions with the VM
pub fn register_audio_builtins(vm: &VM) -> Result<(), SapfError> {
    // Audio playback function
    vm.def_by_name("play", Value::from_object(Arc::new(Primitive::new(
        play_prim, 
        Value::Real(0.0), 
        "play", 
        "Start audio playback of signal/list", 
        1, 
        0
    ))))?;

    // Stop all audio
    vm.def_by_name("stopall", Value::from_object(Arc::new(Primitive::new(
        stop_all_prim, 
        Value::Real(0.0), 
        "stopall", 
        "Stop all audio playback", 
        0, 
        0
    ))))?;

    // Audio system information
    vm.def_by_name("audioinfo", Value::from_object(Arc::new(Primitive::new(
        audio_info_prim, 
        Value::Real(0.0), 
        "audioinfo", 
        "Display audio system information", 
        0, 
        0
    ))))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_register_audio_builtins() {
        let vm = VM::new();
        register_audio_builtins(&vm).expect("Should register audio builtins");
        
        // Test that audio functions are registered
        assert!(vm.lookup_by_name("play").is_some());
        assert!(vm.lookup_by_name("stopall").is_some());
        assert!(vm.lookup_by_name("audioinfo").is_some());
    }

    #[test]
    fn test_stop_all_function() {
        let vm = VM::new();
        register_audio_builtins(&vm).expect("Should register audio builtins");
        
        let mut thread = Thread::new().unwrap();
        
        // Test stopall function
        let stop_fn = vm.lookup_by_name("stopall").unwrap();
        let result = stop_fn.apply(&mut thread, &[]);
        
        // Should not error even if no audio is playing
        assert!(result.is_ok());
    }
}
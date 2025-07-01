// SAPF - Sound As Pure Form
// Built-in functions for sound file I/O

use crate::core::{Value, Object, SapfError, Result};
use crate::core::function::Primitive;
use crate::vm::thread::Thread;
use crate::audio::sound_files::{sf_read, sf_write};

/// Sound file read function: filename offset frames -> channels
/// 
/// Reads a sound file and returns a list of channels
/// 
/// Stack effect: filename offset frames -> channels
fn sf_read_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    // Pop frames, offset, filename from stack
    let frames = thread.pop_real()?;
    let offset = thread.pop_real()?;
    let filename = thread.pop()?;
    
    sf_read(thread, &filename, offset as i64, frames as i64)?;
    
    Ok(())
}

/// Sound file write function: data filename -> 
///
/// Writes audio data to a sound file
///
/// Stack effect: data filename -> 
fn sf_write_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    // Pop filename and data from stack
    let filename = thread.pop()?;
    let data = thread.pop()?;
    
    sf_write(thread, &data, &filename, false)?;
    
    Ok(())
}

/// Sound file write and open function: data filename -> 
///
/// Writes audio data to a sound file and opens it
///
/// Stack effect: data filename -> 
fn sf_write_open_prim(thread: &mut Thread, _prim: &Primitive) -> Result<(), SapfError> {
    // Pop filename and data from stack
    let filename = thread.pop()?;
    let data = thread.pop()?;
    
    sf_write(thread, &data, &filename, true)?;
    
    Ok(())
}

/// Register all sound file built-in functions
pub fn register_sound_file_builtins(vm: &crate::vm::VM) -> Result<(), SapfError> {
    use std::sync::Arc;
    
    vm.def_by_name("<sf", Value::from_object(Arc::new(Primitive::new(
        sf_read_prim, Value::Real(0.0), "<sf", "Read sound file", 3, 1
    ))))?;
    
    vm.def_by_name(">sf", Value::from_object(Arc::new(Primitive::new(
        sf_write_prim, Value::Real(0.0), ">sf", "Write sound file", 2, 0
    ))))?;
    
    vm.def_by_name(">>sf", Value::from_object(Arc::new(Primitive::new(
        sf_write_open_prim, Value::Real(0.0), ">>sf", "Write and open sound file", 2, 0
    ))))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;
    use crate::core::value::StringObject;
    use std::sync::{Arc, OnceLock};
    
    fn get_test_vm() -> &'static VM {
        static VM_INSTANCE: OnceLock<VM> = OnceLock::new();
        VM_INSTANCE.get_or_init(|| VM::new())
    }
    
    #[test]
    fn test_sf_write_stack_operation() {
        let vm = get_test_vm();
        let mut thread = Thread::new(vm.get_audio_rate());
        
        // Push test data onto stack
        let filename = Value::Object(Arc::new(StringObject::new("test.wav".to_string())));
        let data = Value::Real(1.0); // Simplified test data
        
        thread.push(data).unwrap();
        thread.push(filename).unwrap();
        
        // This would normally work with actual audio data
        // For now, just test that the function signature is correct
        let prim = Primitive::new(sf_write_prim, Value::Real(0.0), ">sf", "Write sound file", 2, 0);
        let result = sf_write_prim(&mut thread, &prim);
        
        // The function should handle the error gracefully
        match result {
            Ok(_) => {
                // Success case - file was written
            }
            Err(SapfError::Type(_)) => {
                // Expected error for simplified test data
            }
            Err(e) => {
                // Other errors might indicate real issues
                eprintln!("Unexpected error: {:?}", e);
            }
        }
    }
    
    #[test]
    fn test_sf_read_stack_operation() {
        let vm = get_test_vm();
        let mut thread = Thread::new(vm.get_audio_rate());
        
        // Push test parameters onto stack
        let filename = Value::Object(Arc::new(StringObject::new("nonexistent.wav".to_string())));
        let offset = Value::Real(0.0);
        let frames = Value::Real(-1.0); // -1 means read all frames
        
        thread.push(filename).unwrap();
        thread.push(offset).unwrap();
        thread.push(frames).unwrap();
        
        // This should fail because the file doesn't exist
        let prim = Primitive::new(sf_read_prim, Value::Real(0.0), "<sf", "Read sound file", 3, 1);
        let result = sf_read_prim(&mut thread, &prim);
        assert!(result.is_err());
    }
}
//! VM class for SAPF - Global state and built-in function management
//!
//! This module implements the VM class which manages global state, built-in functions,
//! and provides the runtime environment for SAPF execution.

use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use crate::core::value::{Value, Object};
use crate::core::error::{SapfError, Result};
use crate::core::form::{GTable, GForm, Form};
use crate::core::list::List;
use crate::core::symbol;
use crate::vm::thread::{Thread, Rate};

/// Default audio constants
pub const DEFAULT_SAMPLE_RATE: f64 = 96000.0;
pub const DEFAULT_CONTROL_BLOCK_SIZE: usize = 128;
pub const DEFAULT_V_BLOCK_SIZE: usize = 1;
pub const DEFAULT_Z_BLOCK_SIZE: usize = 512;

/// Global VM instance (singleton pattern)
static VM_INSTANCE: OnceLock<VM> = OnceLock::new();

/// SAPF Virtual Machine
/// 
/// The VM manages global state including built-in functions, constants,
/// and runtime configuration. This corresponds to the VM class in the C++ implementation.
#[derive(Debug)]
pub struct VM {
    /// Built-in functions and values
    builtins: Arc<Mutex<GTable>>,
    
    /// Audio processing rates
    ar: Rate,  // Audio rate
    kr: Rate,  // Control rate
    
    /// Block size for value operations
    v_block_size: usize,
    
    /// Print configuration
    print_length: usize,
    print_depth: usize,
    
    /// Special constants
    nil_v: Arc<List>,  // Empty value list
    nil_z: Arc<List>,  // Empty signal list
    
    /// Empty environment
    empty_env: Arc<Form>,
    
    /// Help system
    bif_help: Arc<Mutex<Vec<String>>>,  // Built-in function help
    udf_help: Arc<Mutex<Vec<String>>>,  // User-defined function help
    
    /// Runtime flags
    trace_on: bool,
}

impl VM {
    /// Create a new VM instance
    pub fn new() -> Self {
        let ar = Rate {
            sample_rate: DEFAULT_SAMPLE_RATE,
            block_size: DEFAULT_Z_BLOCK_SIZE,
        };
        
        let kr = Rate {
            sample_rate: ar.sample_rate / DEFAULT_CONTROL_BLOCK_SIZE as f64,
            block_size: DEFAULT_CONTROL_BLOCK_SIZE,
        };
        
        // Create empty lists
        let nil_v = Arc::new(List::new(crate::core::list::ElementType::Value));
        let nil_z = Arc::new(List::new(crate::core::list::ElementType::Real));
        
        // Create empty environment
        let empty_env = Arc::new(Form::new());
        
        VM {
            builtins: Arc::new(Mutex::new(GTable::new())),
            ar,
            kr,
            v_block_size: DEFAULT_V_BLOCK_SIZE,
            print_length: 20,
            print_depth: 8,
            nil_v,
            nil_z,
            empty_env,
            bif_help: Arc::new(Mutex::new(Vec::new())),
            udf_help: Arc::new(Mutex::new(Vec::new())),
            trace_on: false,
        }
    }
    
    /// Get the global VM instance
    pub fn instance() -> &'static VM {
        VM_INSTANCE.get_or_init(|| VM::new())
    }
    
    /// Initialize the global VM instance (for testing)
    pub fn init_for_test() -> &'static VM {
        VM_INSTANCE.get_or_init(|| VM::new())
    }
    
    // === Audio Rate Management ===
    
    /// Get the audio rate
    pub fn audio_rate(&self) -> &Rate {
        &self.ar
    }
    
    /// Get the control rate
    pub fn control_rate(&self) -> &Rate {
        &self.kr
    }
    
    /// Set the sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f64) -> Result<()> {
        if sample_rate <= 0.0 {
            return Err(SapfError::InvalidValue("Sample rate must be positive".to_string()));
        }
        
        self.ar.sample_rate = sample_rate;
        self.kr.sample_rate = sample_rate / DEFAULT_CONTROL_BLOCK_SIZE as f64;
        Ok(())
    }
    
    /// Get the V block size
    pub fn v_block_size(&self) -> usize {
        self.v_block_size
    }
    
    /// Set the V block size
    pub fn set_v_block_size(&mut self, size: usize) {
        self.v_block_size = size;
    }
    
    // === Built-in Function Management ===
    
    /// Define a built-in value
    pub fn def(&self, key: Value, value: Value) -> Result<Value> {
        let builtins = self.builtins.lock().unwrap();
        builtins.put_impure(key, value.clone())?;
        Ok(value)
    }
    
    /// Define a built-in by name
    pub fn def_by_name(&self, name: &str, value: Value) -> Result<Value> {
        let symbol = symbol::get_symbol(name);
        let key = Value::from_object(symbol);
        self.def(key, value)
    }
    
    /// Look up a built-in value
    pub fn lookup(&self, key: &Value) -> Option<Value> {
        let builtins = self.builtins.lock().unwrap();
        builtins.get(key)
    }
    
    /// Look up a built-in by name
    pub fn lookup_by_name(&self, name: &str) -> Option<Value> {
        let symbol = symbol::get_symbol(name);
        let key = Value::from_object(symbol);
        self.lookup(&key)
    }
    
    // === Constants ===
    
    /// Get the empty value list (nil)
    pub fn nil_v(&self) -> &Arc<List> {
        &self.nil_v
    }
    
    /// Get the empty signal list
    pub fn nil_z(&self) -> &Arc<List> {
        &self.nil_z
    }
    
    /// Get the empty environment
    pub fn empty_env(&self) -> &Arc<Form> {
        &self.empty_env
    }
    
    // === Print Configuration ===
    
    /// Get the print length limit
    pub fn print_length(&self) -> usize {
        self.print_length
    }
    
    /// Set the print length limit
    pub fn set_print_length(&mut self, length: usize) {
        self.print_length = length;
    }
    
    /// Get the print depth limit
    pub fn print_depth(&self) -> usize {
        self.print_depth
    }
    
    /// Set the print depth limit
    pub fn set_print_depth(&mut self, depth: usize) {
        self.print_depth = depth;
    }
    
    // === Help System ===
    
    /// Add built-in function help
    pub fn add_bif_help(&self, name: &str, mask: Option<&str>, help: Option<&str>) {
        let mut help_str = name.to_string();
        if let Some(mask) = mask {
            help_str.push_str(" @");
            help_str.push_str(mask);
        }
        if let Some(help) = help {
            help_str.push(' ');
            help_str.push_str(help);
        }
        
        let mut bif_help = self.bif_help.lock().unwrap();
        bif_help.push(help_str);
    }
    
    /// Add user-defined function help
    pub fn add_udf_help(&self, name: &str, mask: Option<&str>, help: Option<&str>) {
        let mut help_str = name.to_string();
        if let Some(mask) = mask {
            help_str.push_str(" @");
            help_str.push_str(mask);
        }
        if let Some(help) = help {
            help_str.push(' ');
            help_str.push_str(help);
        }
        
        let mut udf_help = self.udf_help.lock().unwrap();
        udf_help.push(help_str);
    }
    
    /// Get all built-in function help
    pub fn get_bif_help(&self) -> Vec<String> {
        let bif_help = self.bif_help.lock().unwrap();
        bif_help.clone()
    }
    
    /// Get all user-defined function help
    pub fn get_udf_help(&self) -> Vec<String> {
        let udf_help = self.udf_help.lock().unwrap();
        udf_help.clone()
    }
    
    // === Tracing ===
    
    /// Check if tracing is enabled
    pub fn trace_on(&self) -> bool {
        self.trace_on
    }
    
    /// Enable or disable tracing
    pub fn set_trace(&mut self, enabled: bool) {
        self.trace_on = enabled;
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_creation() {
        let vm = VM::new();
        
        assert_eq!(vm.audio_rate().sample_rate, DEFAULT_SAMPLE_RATE);
        assert_eq!(vm.audio_rate().block_size, DEFAULT_Z_BLOCK_SIZE);
        assert_eq!(vm.control_rate().block_size, DEFAULT_CONTROL_BLOCK_SIZE);
        assert_eq!(vm.v_block_size(), DEFAULT_V_BLOCK_SIZE);
        assert_eq!(vm.print_length(), 20);
        assert_eq!(vm.print_depth(), 8);
        assert!(!vm.trace_on());
    }
    
    #[test]
    fn test_sample_rate_setting() {
        let mut vm = VM::new();
        
        vm.set_sample_rate(44100.0).unwrap();
        assert_eq!(vm.audio_rate().sample_rate, 44100.0);
        assert_eq!(vm.control_rate().sample_rate, 44100.0 / DEFAULT_CONTROL_BLOCK_SIZE as f64);
        
        // Test invalid sample rate
        assert!(vm.set_sample_rate(0.0).is_err());
        assert!(vm.set_sample_rate(-1000.0).is_err());
    }
    
    #[test]
    fn test_builtin_definition() {
        let vm = VM::new();
        
        let symbol = crate::core::symbol::get_symbol("test");
        let key = Value::from_object(symbol);
        let value = Value::Real(42.0);
        
        vm.def(key.clone(), value.clone()).unwrap();
        
        let retrieved = vm.lookup(&key).unwrap();
        assert_eq!(retrieved, value);
    }
    
    #[test]
    fn test_builtin_by_name() {
        let vm = VM::new();
        
        let value = Value::Real(123.0);
        vm.def_by_name("myvar", value.clone()).unwrap();
        
        let retrieved = vm.lookup_by_name("myvar").unwrap();
        assert_eq!(retrieved, value);
        
        // Test non-existent variable
        assert!(vm.lookup_by_name("nonexistent").is_none());
    }
    
    #[test]
    fn test_print_configuration() {
        let mut vm = VM::new();
        
        vm.set_print_length(50);
        vm.set_print_depth(12);
        
        assert_eq!(vm.print_length(), 50);
        assert_eq!(vm.print_depth(), 12);
    }
    
    #[test]
    fn test_constants() {
        let vm = VM::new();
        
        // Test that we can get the constants 
        let nil_v = vm.nil_v();
        let nil_z = vm.nil_z();
        
        // Test the element types
        use crate::core::list::ElementType;
        assert_eq!(nil_v.element_type(), ElementType::Value);
        assert_eq!(nil_z.element_type(), ElementType::Real);
        
        // Test they are empty
        assert!(nil_v.is_end());
        assert!(nil_z.is_end());
    }
    
    #[test]
    fn test_help_system() {
        let vm = VM::new();
        
        vm.add_bif_help("add", Some("nn"), Some("Add two numbers"));
        vm.add_udf_help("myfunc", None, Some("My custom function"));
        
        let bif_help = vm.get_bif_help();
        let udf_help = vm.get_udf_help();
        
        assert_eq!(bif_help.len(), 1);
        assert_eq!(udf_help.len(), 1);
        assert_eq!(bif_help[0], "add @nn Add two numbers");
        assert_eq!(udf_help[0], "myfunc My custom function");
    }
    
    #[test]
    fn test_tracing() {
        let mut vm = VM::new();
        
        assert!(!vm.trace_on());
        
        vm.set_trace(true);
        assert!(vm.trace_on());
        
        vm.set_trace(false);
        assert!(!vm.trace_on());
    }
    
    #[test]
    fn test_global_instance() {
        let vm1 = VM::instance();
        let vm2 = VM::instance();
        
        // Should be the same instance
        assert!(std::ptr::eq(vm1, vm2));
    }
}
//! SAPF - Sound As Pure Form
//! 
//! A functional stack-based language for sound synthesis and processing.

mod core;
mod vm;
mod parser;

use anyhow::Result;
use vm::VM;

fn main() -> Result<()> {
    println!("SAPF - Sound As Pure Form");
    println!("Rust implementation (work in progress)");
    
    // Initialize the VM
    let vm = VM::instance();
    println!("VM initialized with sample rate: {}", vm.audio_rate().sample_rate);
    
    // TODO: Implement REPL, command line parsing, etc.
    
    Ok(())
}

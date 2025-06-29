//! SAPF - Sound As Pure Form
//! 
//! A functional stack-based language for sound synthesis and processing.

mod core;
mod vm;
mod parser;
mod repl;
mod dsp;

use anyhow::Result;
use clap::{Arg, Command};
use std::fs;
use std::path::Path;
use vm::{VM, Thread};
use parser::{lexer::Lexer, parser::Parser};
use repl::Repl;

fn main() -> Result<()> {
    let matches = Command::new("sapf")
        .version("0.1.0")
        .author("SAPF Rust Port Contributors")
        .about("Sound As Pure Form - A functional stack-based language for sound synthesis")
        .arg(
            Arg::new("sample-rate")
                .short('r')
                .long("sample-rate")
                .value_name("RATE")
                .help("Set audio sample rate (default: 96000)")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("prelude")
                .short('p')
                .long("prelude")
                .value_name("FILE")
                .help("Load prelude file on startup")
        )
        .arg(
            Arg::new("batch")
                .short('b')
                .long("batch")
                .help("Run in batch mode (non-interactive)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("test")
                .long("test")
                .help("Run parser tests and exit")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .help("SAPF file to execute")
                .index(1)
        )
        .get_matches();

    // Initialize the VM with optional sample rate
    let _sample_rate = matches.get_one::<u32>("sample-rate").copied().unwrap_or(96000);
    let vm = VM::instance();
    
    // Set sample rate if provided (this would require extending VM API)
    println!("SAPF - Sound As Pure Form");
    println!("Rust Implementation v0.1.0");
    println!("VM initialized with sample rate: {} Hz", vm.audio_rate().sample_rate);
    
    // Handle test mode
    if matches.get_flag("test") {
        return test_parser();
    }
    
    // Load prelude file if specified
    if let Some(prelude_file) = matches.get_one::<String>("prelude") {
        load_file(prelude_file)?;
    }
    
    // Handle file input
    if let Some(file_path) = matches.get_one::<String>("file") {
        return execute_file(file_path, matches.get_flag("batch"));
    }
    
    // Enter interactive REPL mode
    let mut repl = Repl::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize REPL: {:?}", e))?;
    
    repl.run()
        .map_err(|e| anyhow::anyhow!("REPL error: {:?}", e))?;
    
    Ok(())
}

/// Load and execute a SAPF file
fn load_file(file_path: &str) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?;
    
    let mut thread = Thread::new();
    execute_code(&content, &mut thread)?;
    
    println!("Loaded prelude: {}", file_path);
    Ok(())
}

/// Execute a SAPF file
fn execute_file(file_path: &str, batch_mode: bool) -> Result<()> {
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }
    
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", file_path, e))?;
    
    let mut thread = Thread::new();
    execute_code(&content, &mut thread)?;
    
    if !batch_mode {
        println!("Execution complete. Final stack:");
        thread.print_stack();
    }
    
    Ok(())
}

/// Execute SAPF code string
fn execute_code(code: &str, thread: &mut Thread) -> Result<()> {
    // Tokenize
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()
        .map_err(|e| anyhow::anyhow!("Lexer error: {:?}", e))?;
    
    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| anyhow::anyhow!("Parser error: {:?}", e))?;
    
    // Execute
    parser.execute(&ast, thread)
        .map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;
    
    Ok(())
}

fn test_parser() -> Result<()> {
    println!("\n=== Testing Parser ===");
    
    // Test simple expressions
    let test_cases = vec![
        "42",
        "\"hello world\"",
        "[1 2 3]",
        "{a 1 b 2}",
        r"\x [x 2 *]",
    ];
    
    for (i, code) in test_cases.iter().enumerate() {
        println!("Test {}: {}", i + 1, code);
        
        // Tokenize
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().map_err(|e| anyhow::anyhow!("Lexer error: {:?}", e))?;
        println!("  Tokens: {} generated", tokens.len() - 1); // -1 for EOF
        
        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| anyhow::anyhow!("Parser error: {:?}", e))?;
        println!("  AST nodes: {}", ast.len());
        
        // Execute (simplified - just push to stack)
        let mut thread = Thread::new();
        parser.execute(&ast, &mut thread).map_err(|e| anyhow::anyhow!("Execution error: {:?}", e))?;
        println!("  Stack depth after execution: {}", thread.stack_depth());
        
        println!();
    }
    
    Ok(())
}

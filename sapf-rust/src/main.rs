//! SAPF - Sound As Pure Form
//! 
//! A functional stack-based language for sound synthesis and processing.

mod core;
mod vm;
mod parser;

use anyhow::Result;
use vm::{VM, Thread};
use parser::{lexer::Lexer, parser::Parser};

fn main() -> Result<()> {
    println!("SAPF - Sound As Pure Form");
    println!("Rust implementation (work in progress)");
    
    // Initialize the VM
    let vm = VM::instance();
    println!("VM initialized with sample rate: {}", vm.audio_rate().sample_rate);
    
    // Test the parser
    test_parser()?;
    
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

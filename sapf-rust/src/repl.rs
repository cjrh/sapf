//! SAPF REPL (Read-Eval-Print Loop)
//!
//! Interactive command-line interface for SAPF

use crate::core::error::{SapfError, Result};
use crate::vm::{VM, Thread};
use crate::parser::{lexer::Lexer, parser::Parser};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::sync::Arc;

/// SAPF REPL state
pub struct Repl {
    editor: DefaultEditor,
    thread: Thread,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new() -> RustylineResult<Self> {
        let mut editor = DefaultEditor::new()?;
        
        // Load history if available
        let _ = editor.load_history("sapf_history.txt");
        
        Ok(Self {
            editor,
            thread: Thread::new(),
        })
    }
    
    /// Run the REPL main loop
    pub fn run(&mut self) -> Result<()> {
        println!("SAPF - Sound As Pure Form");
        println!("Rust Implementation - Interactive Mode");
        println!("Type 'quit' or press Ctrl+C to exit");
        println!("Type 'help' for assistance");
        println!();
        
        // Initialize VM
        let vm = VM::instance();
        println!("VM initialized with sample rate: {}", vm.audio_rate().sample_rate);
        println!();
        
        loop {
            let readline = self.editor.readline("sapf> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    
                    // Skip empty lines
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Add to history
                    let _ = self.editor.add_history_entry(line);
                    
                    // Handle special commands
                    match line {
                        "quit" | "exit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "help" => {
                            self.show_help();
                            continue;
                        }
                        "clear" => {
                            self.thread.clear_stack();
                            println!("Stack cleared");
                            continue;
                        }
                        "stack" | "st" => {
                            self.show_stack();
                            continue;
                        }
                        "vm" => {
                            self.show_vm_info();
                            continue;
                        }
                        _ => {}
                    }
                    
                    // Process SAPF code
                    if let Err(e) = self.eval(line) {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        
        // Save history
        let _ = self.editor.save_history("sapf_history.txt");
        
        Ok(())
    }
    
    /// Evaluate a line of SAPF code
    fn eval(&mut self, code: &str) -> Result<()> {
        // Tokenize
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize()
            .map_err(|e| SapfError::ParseError(format!("Lexer error: {:?}", e)))?;
        
        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| SapfError::ParseError(format!("Parser error: {:?}", e)))?;
        
        // Execute
        parser.execute(&ast, &mut self.thread)
            .map_err(|_| SapfError::Failed)?;
        
        // Show result if stack is not empty
        if self.thread.stack_depth() > 0 {
            self.show_top_of_stack();
        }
        
        Ok(())
    }
    
    /// Show the top value on the stack
    fn show_top_of_stack(&self) {
        if let Ok(value) = self.thread.peek(0) {
            println!("{}", value.print());
        }
    }
    
    /// Show the entire stack
    fn show_stack(&self) {
        if self.thread.stack_depth() == 0 {
            println!("Stack is empty");
        } else {
            println!("Stack (top to bottom):");
            println!("{}", self.thread.print_stack());
        }
    }
    
    /// Show VM information
    fn show_vm_info(&self) {
        let vm = VM::instance();
        println!("VM Information:");
        println!("  Sample rate: {} Hz", vm.audio_rate().sample_rate);
        println!("  Block size: {} samples", vm.audio_rate().block_size);
        println!("  Stack depth: {}", self.thread.stack_depth());
    }
    
    /// Show help information
    fn show_help(&self) {
        println!("SAPF REPL Commands:");
        println!("  help         - Show this help message");
        println!("  quit, exit   - Exit the REPL");
        println!("  clear        - Clear the stack");
        println!("  stack, st    - Show the current stack");
        println!("  vm           - Show VM information");
        println!();
        println!("SAPF Language:");
        println!("  Numbers:     42, 3.14, 0xFF, 2pi");
        println!("  Strings:     \"hello world\"");
        println!("  Lists:       [1 2 3]");
        println!("  Forms:       {{a 1 b 2}}");
        println!("  Functions:   \\x [x 2 *]");
        println!("  Operations:  +, -, *, /, dup, swap");
        println!();
        println!("Examples:");
        println!("  42           - Push number to stack");
        println!("  1 2 +        - Add two numbers");
        println!("  [1 2 3]      - Create a list");
        println!("  dup          - Duplicate top of stack");
        println!("  stack        - Show current stack");
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
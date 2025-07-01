# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SAPF (Sound As Pure Form) is a functional stack-based language for sound synthesis and processing. This repository contains both the original C++ implementation and a modern Rust port that provides equivalent functionality with improved memory safety and performance.

## Development Commands

### Rust Implementation (Primary)
```bash
cd sapf-rust/

# Build the project
cargo build

# Run tests
cargo test

# Run specific module tests
cargo test <module_name>

# Run the SAPF interpreter
cargo run

# Run with specific options
cargo run -- -r 48000        # Set sample rate to 48kHz
cargo run -- -p prelude.txt  # Load prelude file
cargo run -- --test          # Run parser tests

# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

### C++ Implementation (Legacy)
The original C++ implementation is located in `src/` and `include/` directories. It uses Xcode project files for macOS builds.

## Codebase Architecture

### Rust Implementation Structure (`sapf-rust/src/`)

**Core Language Infrastructure:**
- `core/`: Foundation types (`Value`, `Object` trait, `Form`, `List`, `Function`)
  - `value.rs`: Central value system (Real numbers + Object references)
  - `form.rs`: Key-value mappings with multiple inheritance
  - `function.rs`: Function definitions and primitive operations
  - `list.rs`: Dynamic arrays for both values and audio samples
  - `symbol.rs`: Symbol table for efficient name lookups
  - `error.rs`: Comprehensive error handling

**Virtual Machine:**
- `vm/`: Stack-based execution engine
  - `vm.rs`: Global VM singleton managing built-ins and configuration
  - `thread.rs`: Execution context with data/local stacks
  - `opcode.rs`: Bytecode instruction set
  - `builtins.rs`: Built-in function registry (~85+ functions)

**Language Processing:**
- `parser/`: Complete parsing pipeline
  - `lexer.rs`: Tokenization with position tracking
  - `parser.rs`: Recursive descent parser generating ASTs
  - `codegen.rs`: Bytecode generation and direct interpretation

**Audio Processing:**
- `dsp/`: Digital signal processing
  - `ugen.rs`: Unit Generator base trait for audio generation
  - `oscillators.rs`: Basic audio oscillators
  - `filters.rs`: Audio filtering components
- `audio/`: Real-time I/O using `cpal` library

**User Interface:**
- `repl.rs`: Interactive command-line interface

### Key Architectural Patterns

**Stack-Based Virtual Machine:**
- Dual-stack architecture (data stack + local variables)
- Postfix notation execution (e.g., `5 3 +` → `8`)
- Function calls via stack manipulation

**Object System:**
```rust
enum Value {
    Real(f64),                    // Immediate values
    Object(Arc<dyn Object>),      // Heap objects
    Nil,                         // Empty value
}
```

**Audio Processing:**
- Block-based computation for real-time performance
- Dual-rate processing (audio rate ~96kHz, control rate ~750Hz)
- Multi-channel expansion for stereo/surround processing

**Memory Management:**
- `Arc<dyn Object>` for shared ownership
- Thread-safe access patterns throughout
- No garbage collection - deterministic cleanup

## Development Workflow

### Testing Strategy
The codebase uses comprehensive testing with 190+ tests covering:
- Core type operations and mathematical functions
- VM execution and stack manipulation
- Parser correctness and error handling
- Audio processing components
- Integration between modules

Run tests frequently during development:
```bash
cargo test                    # All tests
cargo test core::             # Core module tests
cargo test vm::               # VM tests
cargo test parser::           # Parser tests
```

### Language Examples
The SAPF language uses postfix notation:
```sapf
; Basic arithmetic
5 3 +                         ; → 8
2 3 * 4 +                     ; → 10

; Stack manipulation
10 3 ba                       ; ba = swap → 3 10
42 aa                         ; aa = dup → 42 42

; Lists and audio
[300 301] 0 saw .3 * play     ; Stereo sawtooth at 300/301 Hz
800 0 sinosc .3 * play        ; Sine wave at 800 Hz
```

### Key Files for Understanding

**Language Specification:**
- `README.txt`: Complete language reference and philosophy
- `sapf-examples.txt`: Comprehensive usage examples
- `sapf-prelude.txt`: Standard library definitions

**Implementation Status:**
- `sapf-rust/TODO.txt`: Detailed conversion progress and architecture notes
- `conversion-cpp-to-rust-prompt.txt`: Conversion strategy and decisions

## Important Notes

### Threading and Safety
The Rust implementation uses `Arc<T>` and `Arc<Mutex<T>>` for thread safety. Audio processing requires careful attention to real-time constraints.

### Audio Configuration
- Default sample rate: 96000 Hz
- Default control rate: 750 Hz
- Configurable block sizes for different processing contexts

### Error Handling
Uses `Result<T, SapfError>` throughout with structured error types. Audio processing paths have specific error recovery strategies.

### Performance Considerations
- Lookup tables for trigonometric functions (`math.rs`)
- Hash caching in string objects
- Block-based audio processing for efficiency
- SIMD-friendly data layouts where applicable

## Current Status

The Rust implementation is ~90% complete with a fully functional REPL and core language features. Audio playback, mathematical operations, stack manipulation, and list processing are all working. The system can execute complex SAPF programs and provides real-time audio synthesis capabilities.
# V8-like JavaScript Engine

> A modern JavaScript engine implementation in Rust featuring tiered compilation, hidden classes, inline caches, and generational garbage collection.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-in%20development-yellow.svg)](IMPLEMENTATION_ROADMAP.md)

## 🚀 Overview

This project implements a V8-inspired JavaScript engine in Rust, demonstrating modern JavaScript virtual machine techniques including:

- **Tiered Compilation**: Interpreter → Baseline JIT → Optimizing JIT
- **Hidden Classes (Shapes)**: Fast property access through object shape optimization
- **Inline Caches**: Adaptive optimization based on runtime type information
- **Generational GC**: Efficient memory management with young/old generation separation
- **Advanced Optimizations**: Speculative compilation with deoptimization support

### 🎯 Project Goals

- **Educational**: Demonstrate modern JS engine implementation techniques
- **Performance**: Achieve competitive performance through advanced optimizations
- **Memory Safety**: Leverage Rust's safety guarantees for VM implementation
- **Modular Architecture**: Clean, extensible design for research and experimentation

## 🏗️ Architecture

```
JavaScript Source Code
         ↓
    Lexer/Parser → AST
         ↓
   Bytecode Generator
         ↓
  ┌─────────────────┐
  │   Interpreter   │ ←──── Profiler & Inline Caches
  │    (Ignition)   │
  └─────────────────┘
         ↓ (hot functions)
  ┌─────────────────┐
  │  Baseline JIT   │ ←──── Cranelift Backend
  │   (Cranelift)   │
  └─────────────────┘
         ↓ (very hot functions)
  ┌─────────────────┐
  │ Optimizing JIT  │ ←──── Sea of Nodes IR
  │  (TurboFan-like)│       Speculative Optimizations
  └─────────────────┘
         ↕ (deoptimization)
  ┌─────────────────┐
  │ Deoptimization  │
  │     System      │
  └─────────────────┘
```

### Core Components

- **Frontend**: Lexer and recursive descent parser with Pratt parsing for expressions
- **Bytecode**: Stack-based virtual machine with accumulator (Ignition-style)
- **Runtime**: JavaScript value system with hidden classes and inline caches
- **JIT**: Two-tier compilation using Cranelift for code generation
- **GC**: Generational garbage collector with copying young space and mark-sweep old space

## 🚦 Current Status

### ✅ Implemented (Phase 1: Frontend)
- **Lexer**: Complete tokenization of JavaScript constructs
  - Numbers (integer, float, exponential), strings with escape sequences
  - All operators (arithmetic, logical, bitwise, comparison, assignment)
  - Keywords, identifiers with Unicode support
  - Comments (single-line `//` and block `/* */`)
- **Parser**: Basic recursive descent parser with AST generation
  - Expression parsing infrastructure
  - Error recovery and diagnostics with source positions
- **CLI**: REPL mode and file execution support
- **Error Handling**: Comprehensive error reporting with colored terminal output

### 🔄 In Development (Phase 2: Bytecode & VM)
- Bytecode instruction set design
- AST to bytecode compiler
- Stack-based virtual machine interpreter
- Basic value system and objects

### 📋 Planned Features
- **Phase 3**: Hidden classes and object system
- **Phase 4**: Inline caches for adaptive optimization  
- **Phase 5**: JIT compilation with Cranelift
- **Phase 6**: Generational garbage collection
- **Phase 7**: Advanced optimizations and deoptimization
- **Phase 8**: Extended JavaScript language support

## 🔧 Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Installation

```bash
git clone <repository-url>
cd v8
cargo build
```

### Usage

#### REPL Mode
```bash
cargo run
```

Example session:
```
V8-like JavaScript Engine v0.1.0
Type 'exit' to quit
js> console.log("Hello, World!")
Successfully parsed source code
js> exit
```

#### Execute JavaScript Files
```bash
cargo run examples/hello.js
```

#### Run Tests
```bash
cargo test                    # Run all tests
cargo test --lib             # Library unit tests only
cargo test --test '*'        # Integration tests only
```

### Development Commands

```bash
# Build and run
cargo build                   # Compile the project
cargo run                     # REPL mode
cargo run <file.js>          # Execute JavaScript file
cargo build --release        # Optimized build

# Code quality
cargo check                   # Quick syntax check
cargo clippy                  # Rust linter
cargo fmt                     # Format code

# Testing
cargo test                    # Run all tests
cargo test golden_tests       # Bytecode verification tests
cargo bench                   # Performance benchmarks
```

## 📚 Technical Details

### Supported JavaScript Features (Current)
- Lexical analysis of all major JavaScript constructs
- Basic expression parsing (work in progress)
- Error reporting with source locations

### Planned JavaScript Support
- Variables: `var`, `let`, `const`
- Functions: declarations and expressions
- Objects: literals and property access
- Arrays: literals and element access
- Control flow: `if/else`, `while`, `for`, `return`
- Operators: arithmetic, logical, comparison, assignment

### Performance Characteristics

The engine is designed for performance through:

- **Fast Property Access**: Hidden classes enable O(1) property lookup
- **Adaptive Optimization**: Inline caches adapt to runtime patterns
- **Efficient Compilation**: Two-tier JIT balances compilation time and code quality
- **Memory Efficiency**: Generational GC minimizes pause times

### Dependencies

- **nom**: Parser combinator library for robust parsing
- **thiserror**: Ergonomic error handling
- **unicode-xid**: JavaScript identifier validation
- **criterion**: Performance benchmarking (dev dependency)

## 🛠️ Development

### Project Structure

```
src/
├── lib.rs              # Main engine interface
├── main.rs            # CLI entry point
├── error/             # Error handling and diagnostics
├── lexer/             # Lexical analysis
└── parser/            # Syntactic analysis and AST
specs/                 # Detailed design documents
├── ARCHITECTURE_PLAN.md
├── IMPLEMENTATION_ROADMAP.md
└── PROJECT.md
```

### Development Workflow

1. **Phase-based Development**: Follow the implementation roadmap phases
2. **Test-Driven**: Write tests for new features
3. **Benchmarking**: Measure performance impact of changes
4. **Documentation**: Update specs and README for significant changes

### Testing Strategy

- **Unit Tests**: Component-specific functionality
- **Golden Tests**: Expected bytecode/output verification  
- **Integration Tests**: End-to-end pipeline testing
- **Performance Tests**: Regression detection and benchmarking
- **Property Testing**: Fuzzing and edge case discovery

### Contributing

Contributions are welcome! Areas of interest:

- **Parser Extensions**: Additional JavaScript language constructs
- **Bytecode Design**: Virtual machine instruction set
- **Optimization**: JIT compilation and runtime optimizations
- **Testing**: Expand test coverage and fuzzing
- **Documentation**: Improve technical documentation

## 📊 Performance Goals

Target performance characteristics:

- **Parsing**: < 1ms per 1000 lines of code
- **Property Access**: < 10ns with inline caches
- **JIT Compilation**: 5-20x speedup over interpreter
- **GC Pauses**: < 10ms for heaps up to 100MB
- **Overall**: Competitive with other modern JS engines

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **V8 Team**: Inspiration and architectural guidance
- **Cranelift**: JIT compilation backend
- **Rust Community**: Language and ecosystem support

## 📖 Additional Resources

- [Implementation Roadmap](IMPLEMENTATION_ROADMAP.md) - Detailed development phases
- [Architecture Plan](specs/ARCHITECTURE_PLAN.md) - Technical architecture details  
- [Project Overview](specs/PROJECT.md) - Comprehensive project documentation
- [CLAUDE.md](CLAUDE.md) - Development guidance and commands

---

**Status**: 🟡 Phase 1 Complete - Lexer and basic parser implemented  
**Next**: 🔄 Phase 2 - Bytecode design and virtual machine  
**Timeline**: 6-9 months to full implementation
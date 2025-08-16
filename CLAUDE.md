# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a V8-like JavaScript engine implementation in Rust, featuring a complete compilation and execution pipeline with advanced optimization techniques. The project demonstrates modern JavaScript engine architecture including tiered compilation, hidden classes, inline caches, and generational garbage collection.

## Common Development Commands

### Building and Running
- `cargo build` - Compile the project
- `cargo run` - Build and run the REPL mode
- `cargo run <file.js>` - Execute a JavaScript file
- `cargo build --release` - Build optimized release version

### Testing and Code Quality
- `cargo test` - Run all tests (unit, integration, and golden tests)
- `cargo test --lib` - Run library unit tests only
- `cargo test --test '*'` - Run integration tests only
- `cargo test golden_tests` - Run golden tests (bytecode/output verification)
- `cargo check` - Quick syntax and type checking without full compilation
- `cargo clippy` - Run Rust linter for code quality suggestions
- `cargo fmt` - Format code according to Rust style guidelines

### Benchmarking and Performance
- `cargo bench` - Run performance benchmarks
- `cargo bench -- --save-baseline <name>` - Save performance baseline
- `cargo test ic_coverage_tests` - Verify inline cache state coverage
- `cargo test performance` - Run performance regression tests

### Development Tools
- `cargo run --bin disassembler <file.js>` - Show bytecode disassembly
- `cargo run --bin profiler <file.js>` - Generate performance profile
- `cargo run --bin heap_dump <file.js>` - Analyze heap usage

### Project Management
- `cargo clean` - Remove build artifacts
- `cargo update` - Update dependencies to latest compatible versions

## Code Architecture

This is a sophisticated JavaScript engine with a modular architecture:

### Core Components
- `src/main.rs` - CLI entry point supporting REPL and file execution
- `src/lib.rs` - Main engine interface and module exports
- `src/error/` - Error handling and diagnostic system
- `src/lexer/` - Lexical analysis (tokenization)
- `src/parser/` - Syntactic analysis (AST generation)

### Key Dependencies
- `nom` - Parser combinator library for robust parsing
- `thiserror` - Ergonomic error handling
- `unicode-xid` - JavaScript identifier validation
- `criterion` - Performance benchmarking framework
- `assert_matches` - Pattern matching assertions for tests
- `proptest` - Property-based testing

### Planned Architecture (per documentation)
```
Frontend (Parser/Lexer) → Bytecode Generator → VM Interpreter
                                                     ↓
Hidden Classes + Inline Caches ← Profiler → JIT Compiler
                                                     ↓
                                          Garbage Collector
```

### Module Structure (Future)
- `rustjs-lexer/` - Lexical analysis
- `rustjs-parser/` - Syntax analysis  
- `rustjs-ir/` - Intermediate representation and bytecode
- `rustjs-runtime/` - Value system, objects, shapes
- `rustjs-vm/` - Virtual machine and interpreter
- `rustjs-jit/` - JIT compilation (Cranelift-based)
- `rustjs-gc/` - Garbage collection
- `rustjs-std/` - JavaScript standard library

## Development Workflow

### Adding Language Features
1. Update lexer (`src/lexer/`) for new tokens
2. Extend parser (`src/parser/`) for new syntax
3. Add AST nodes to `src/parser/ast.rs`
4. Implement bytecode generation
5. Add interpreter support
6. Create comprehensive tests

### Testing Strategy
1. **Unit Tests**: Component-specific functionality
2. **Golden Tests**: Expected bytecode/output verification
3. **Integration Tests**: Cross-component pipeline testing
4. **IC Coverage Tests**: Inline cache state transition verification
5. **Performance Tests**: Regression detection and benchmarking

### Key Testing Files
- `tests/golden/bytecode/` - Expected bytecode output
- `tests/golden/output/` - Expected execution results
- `tests/unit/` - Component unit tests
- `tests/integration/` - End-to-end pipeline tests
- `benches/` - Performance benchmarks

## Important Implementation Notes

### Parser Architecture
- Uses Pratt parsing for expressions with proper precedence
- Recursive descent for statements and declarations
- Comprehensive error recovery and reporting
- Location tracking for debugging information

### Future JIT Integration
- Baseline JIT using Cranelift backend
- Optimizing JIT with Sea of Nodes IR
- Deoptimization system for failed speculations
- On-Stack Replacement (OSR) for hot loops

### Memory Management
- Generational garbage collection design
- Write barriers for cross-generational references
- Precise stack scanning and root marking
- Compaction to reduce fragmentation

### Performance Optimization
- Hidden classes (shapes) for fast property access
- Inline caches for adaptive method/property optimization
- Tiered compilation strategy
- Speculative optimizations with deoptimization fallback

## Code Style and Conventions

- Use descriptive error messages with source locations
- Implement comprehensive test coverage for all features
- Follow Rust naming conventions and idioms
- Document complex algorithms and data structures
- Prefer type safety over performance in initial implementations
- Use `thiserror` for structured error handling
- Implement `Display` and `Debug` for user-facing types
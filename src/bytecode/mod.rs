//! Bytecode system for JavaScript engine
//! 
//! This module implements a stack-based virtual machine with accumulator architecture,
//! similar to V8's Ignition interpreter. The bytecode serves as an intermediate 
//! representation between the AST and machine code.
//!
//! # Architecture
//! 
//! - **Accumulator**: Primary register for operations and results
//! - **Stack**: For temporary values and function parameters  
//! - **Locals**: Indexed access to local variables and parameters
//! - **Constants**: Pool of constants with u16 indices
//!
//! # Instruction Format
//!
//! Instructions are designed to be compact while maintaining readability:
//! - Most instructions are 1-3 bytes
//! - Operands use typed indices (ConstIndex, LocalIndex, etc.)
//! - Jump offsets are signed 16-bit for ±32KB range

pub mod instruction;
pub mod function;
pub mod constant_pool;
pub mod disassembler;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use instruction::*;
pub use function::*;
pub use constant_pool::*;
pub use disassembler::*;
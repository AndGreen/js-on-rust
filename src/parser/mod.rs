//! JavaScript parser
//!
//! Converts a stream of tokens into an Abstract Syntax Tree (AST).
//! Uses recursive descent parsing with Pratt parser for expressions.
//!
//! The parser is decomposed into specialized agents:
//! - `core`: Token management and coordination utilities
//! - `statements`: Statement and declaration parsing
//! - `expressions`: Expression parsing with Pratt parser
//! - `parser`: Main orchestration layer

pub mod ast;
pub mod core;
pub mod expressions;
pub mod parser;
pub mod statements;

#[cfg(test)]
mod tests;

pub use ast::*;
pub use parser::Parser;
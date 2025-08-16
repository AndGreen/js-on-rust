//! JavaScript parser
//!
//! Converts a stream of tokens into an Abstract Syntax Tree (AST).
//! Uses recursive descent parsing with Pratt parser for expressions.

pub mod ast;
pub mod parser;
pub mod pratt;

#[cfg(test)]
mod tests;

pub use ast::*;
pub use parser::Parser;
//! JavaScript lexical analyzer (tokenizer)
//!
//! Converts JavaScript source code into a stream of tokens for parsing.
//! Supports the core JavaScript token types needed for our engine.

pub mod token;
pub mod lexer;

#[cfg(test)]
mod tests;

pub use token::{Token, TokenKind, Keyword};
pub use lexer::Lexer;
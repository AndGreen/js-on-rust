//! Core lexer infrastructure
//!
//! Contains the fundamental building blocks for lexical analysis:
//! position tracking, input handling, and lexer context management.

pub mod position;
pub mod input;
pub mod context;

pub use position::{Position, TokenPosition};
pub use input::Input;
pub use context::{LexerContext, LexerConfig, LexerConfigBuilder, EcmaVersion};
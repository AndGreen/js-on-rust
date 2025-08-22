//! JavaScript lexical analyzer (tokenizer)
//!
//! Provides both legacy monolithic lexer and new modular lexer architecture.
//! The modular system offers better extensibility, performance, and maintainability.

// Legacy lexer (maintained for compatibility)
pub mod token;
pub mod lexer;

// New modular lexer architecture
pub mod core;
pub mod scanners;
pub mod tokens;
pub mod utils;
pub mod new_lexer;

#[cfg(test)]
mod tests;

// Primary exports (backward compatible - used by parser)
pub use token::{Token, TokenKind, Keyword};
pub use lexer::Lexer;

// New modular exports (for new code and future expansion)
pub use new_lexer::{ModularLexer, LexerBuilder, LexerFlags};
pub use core::{LexerContext, LexerConfig, LexerConfigBuilder, EcmaVersion};
pub use tokens::{Token as ModularToken, TokenKind as ModularTokenKind, TokenMetadata, TriviaToken, TokenBuilder};
pub use utils::{LexerValidator, UnicodeHelper, EscapeSequenceParser};

// Legacy aliases for explicit use
pub use token::{Token as LegacyToken, TokenKind as LegacyTokenKind};
pub use lexer::Lexer as LegacyLexer;

// Convenience re-exports
pub use scanners::{
    numbers::NumberLiteral, 
    strings::StringLiteral, 
    identifiers::IdentifierToken, 
    operators::OperatorToken, 
    comments::CommentToken
};
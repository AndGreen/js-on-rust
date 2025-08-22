//! Token system for JavaScript lexer
//!
//! Provides comprehensive token representation with rich metadata,
//! unified token classification, and fast keyword lookup capabilities.

pub mod kinds;
pub mod token;
pub mod keywords;

pub use kinds::TokenKind;
pub use token::{Token, TokenMetadata, TriviaToken, TriviaKind, TokenFlags, TokenBuilder};
pub use keywords::{KeywordSystem, KeywordInfo, KeywordCategory, is_keyword, is_contextual_keyword, is_literal, lookup_keyword, suggest_keyword_corrections};
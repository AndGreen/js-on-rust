//! Token scanners for different JavaScript constructs
//!
//! Contains specialized scanners for different types of tokens:
//! numbers, strings, identifiers, operators, and comments.

use crate::error::{Error, Result, Span};
use crate::lexer::core::{Input, LexerContext};

/// Common interface for token scanners
pub trait Scanner {
    /// Type of token this scanner produces
    type Token;
    
    /// Try to scan a token from the current input position
    /// Returns Some(token) if successful, None if this scanner doesn't handle the current character
    fn try_scan(&mut self, input: &mut Input, context: &LexerContext) -> Option<Result<Self::Token>>;
    
    /// Check if this scanner can handle the current character
    fn can_scan(&self, input: &Input, context: &LexerContext) -> bool;
    
    /// Get the name of this scanner for debugging
    fn name(&self) -> &'static str;
}

/// Result of a scan operation
#[derive(Debug)]
pub enum ScanResult<T> {
    /// Successfully scanned a token
    Token(T),
    /// Current character doesn't match this scanner
    NoMatch,
    /// Error during scanning
    Error(Error),
}

impl<T> ScanResult<T> {
    /// Convert to Option<Result<T>> for easier handling
    pub fn into_option(self) -> Option<Result<T>> {
        match self {
            ScanResult::Token(token) => Some(Ok(token)),
            ScanResult::NoMatch => None,
            ScanResult::Error(err) => Some(Err(err)),
        }
    }
    
    /// Check if this is a successful token
    pub fn is_token(&self) -> bool {
        matches!(self, ScanResult::Token(_))
    }
    
    /// Check if this is a no-match result
    pub fn is_no_match(&self) -> bool {
        matches!(self, ScanResult::NoMatch)
    }
    
    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self, ScanResult::Error(_))
    }
}

/// Trait for scanners that can provide lookahead information
pub trait LookaheadScanner: Scanner {
    /// Check if this scanner would match starting from current position
    /// without consuming any input
    fn would_match(&self, input: &mut Input, context: &LexerContext) -> bool;
    
    /// Get the expected length of the token if this scanner were to match
    /// This is used for optimization and error reporting
    fn expected_length(&self, input: &mut Input, context: &LexerContext) -> Option<usize>;
}

/// Trait for scanners that need to handle state or nesting
pub trait StatefulScanner: Scanner {
    /// Called when entering a new scanning context
    fn enter_context(&mut self, context: &LexerContext);
    
    /// Called when exiting a scanning context
    fn exit_context(&mut self, context: &LexerContext);
    
    /// Reset scanner state
    fn reset(&mut self);
}

/// Helper function to create lexer errors
pub fn lexer_error(message: impl Into<String>, span: Span) -> Error {
    Error::lexer(message.into(), span)
}

/// Helper function to create unexpected character errors
pub fn unexpected_char_error(ch: char, input: &Input) -> Error {
    lexer_error(
        format!("Unexpected character: '{}'", ch),
        input.current_token_span(),
    )
}

/// Helper trait for character classification
pub trait CharClassifier {
    fn is_identifier_start(ch: char) -> bool;
    fn is_identifier_continue(ch: char) -> bool;
    fn is_whitespace(ch: char) -> bool;
    fn is_line_terminator(ch: char) -> bool;
    fn is_decimal_digit(ch: char) -> bool;
    fn is_hex_digit(ch: char) -> bool;
    fn is_octal_digit(ch: char) -> bool;
    fn is_binary_digit(ch: char) -> bool;
}

/// Default character classification following ECMAScript spec
pub struct EcmaCharClassifier;

impl CharClassifier for EcmaCharClassifier {
    fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_start(ch)
    }
    
    fn is_identifier_continue(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '$' || unicode_xid::UnicodeXID::is_xid_continue(ch)
    }
    
    fn is_whitespace(ch: char) -> bool {
        matches!(ch, ' ' | '\t' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}') ||
        (ch as u32 >= 0x1680 && ch.is_whitespace())
    }
    
    fn is_line_terminator(ch: char) -> bool {
        matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
    }
    
    fn is_decimal_digit(ch: char) -> bool {
        matches!(ch, '0'..='9')
    }
    
    fn is_hex_digit(ch: char) -> bool {
        matches!(ch, '0'..='9' | 'a'..='f' | 'A'..='F')
    }
    
    fn is_octal_digit(ch: char) -> bool {
        matches!(ch, '0'..='7')
    }
    
    fn is_binary_digit(ch: char) -> bool {
        matches!(ch, '0' | '1')
    }
}

// Re-export scanner modules
pub mod numbers;
pub mod strings;
pub mod identifiers;
pub mod operators;
pub mod comments;

pub use numbers::NumberScanner;
pub use strings::StringScanner;
pub use identifiers::IdentifierScanner;
pub use operators::OperatorScanner;
pub use comments::CommentScanner;
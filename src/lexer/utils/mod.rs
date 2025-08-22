//! Utility modules for JavaScript lexer
//!
//! Provides common utilities for Unicode handling, escape sequence processing,
//! and comprehensive validation of lexical constructs.

pub mod unicode;
pub mod escape;
pub mod validation;

pub use unicode::{UnicodeHelper, UnicodeCategory, UnicodeError};
pub use escape::{EscapeSequenceParser, EscapeValidator, EscapeSequence};
pub use validation::{LexerValidator, ValidationWarning, ValidationError, ValidationSummary, WarningCategory, ErrorCategory};
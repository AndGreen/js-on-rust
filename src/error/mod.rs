//! Error handling and diagnostics for the JavaScript engine

pub mod diagnostic;

pub use diagnostic::Diagnostic;

/// Source position information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self { start, end, line, column }
    }
    
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Engine error types
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Lexer error: {message} at line {line}, column {column}")]
    Lexer {
        message: String,
        line: u32,
        column: u32,
        span: Span,
    },
    
    #[error("Parser error: {message} at line {line}, column {column}")]
    Parser {
        message: String,
        line: u32,
        column: u32,
        span: Span,
    },
    
    #[error("Runtime error: {message}")]
    Runtime {
        message: String,
        span: Option<Span>,
    },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    pub fn lexer(message: impl Into<String>, span: Span) -> Self {
        Self::Lexer {
            message: message.into(),
            line: span.line,
            column: span.column,
            span,
        }
    }
    
    pub fn parser(message: impl Into<String>, span: Span) -> Self {
        Self::Parser {
            message: message.into(),
            line: span.line,
            column: span.column,
            span,
        }
    }
    
    pub fn runtime(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::Runtime {
            message: message.into(),
            span,
        }
    }
    
    pub fn span(&self) -> Option<Span> {
        match self {
            Error::Lexer { span, .. } => Some(*span),
            Error::Parser { span, .. } => Some(*span),
            Error::Runtime { span, .. } => *span,
            Error::Io(_) => None,
        }
    }
}

/// Result type for engine operations
pub type Result<T> = std::result::Result<T, Error>;
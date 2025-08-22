//! Enhanced token representation
//!
//! Provides a rich token structure with comprehensive metadata including
//! position information, source text, and additional context for advanced
//! language tooling features.

use super::kinds::TokenKind;
use crate::error::Span;
use crate::lexer::{NumberLiteral, StringLiteral, IdentifierToken, OperatorToken, CommentToken};
use std::fmt;

/// A JavaScript token with comprehensive metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token (literal, operator, identifier, etc.)
    pub kind: TokenKind,
    /// Position and span information in source
    pub span: Span,
    /// Original source text that produced this token
    pub text: String,
    /// Additional metadata
    pub metadata: TokenMetadata,
}

/// Additional metadata for tokens
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenMetadata {
    /// Whether this token has leading whitespace
    pub has_leading_whitespace: bool,
    /// Whether this token has trailing whitespace  
    pub has_trailing_whitespace: bool,
    /// Whether this token is on a new line
    pub is_new_line: bool,
    /// Indentation level (for formatting)
    pub indentation_level: usize,
    /// Associated trivia (comments, whitespace) before this token
    pub leading_trivia: Vec<TriviaToken>,
    /// Associated trivia after this token
    pub trailing_trivia: Vec<TriviaToken>,
    /// Token flags for special processing
    pub flags: TokenFlags,
}

/// Trivia tokens (whitespace, comments, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct TriviaToken {
    /// The trivia content
    pub kind: TriviaKind,
    /// Span of the trivia
    pub span: Span,
    /// Original text
    pub text: String,
}

/// Types of trivia
#[derive(Debug, Clone, PartialEq)]
pub enum TriviaKind {
    /// Whitespace (spaces, tabs)
    Whitespace,
    /// Line terminator (\n, \r\n, etc.)
    LineTerminator,
    /// Line comment (// ...)
    LineComment,
    /// Block comment (/* ... */)
    BlockComment,
    /// HTML comment (<!-- ... -->)
    HtmlComment,
}

/// Token processing flags
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TokenFlags {
    /// Token was created during error recovery
    pub is_error_recovery: bool,
    /// Token is synthetic (inserted by parser)
    pub is_synthetic: bool,
    /// Token should be ignored in semantic analysis
    pub is_ignored: bool,
    /// Token marks a potential automatic semicolon insertion point
    pub is_asi_point: bool,
    /// Token contains escape sequences
    pub has_escapes: bool,
    /// Token has Unicode content
    pub has_unicode: bool,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self {
            kind,
            span,
            text,
            metadata: TokenMetadata::default(),
        }
    }
    
    /// Create a token with metadata
    pub fn with_metadata(kind: TokenKind, span: Span, text: String, metadata: TokenMetadata) -> Self {
        Self {
            kind,
            span,
            text,
            metadata,
        }
    }
    
    /// Create a synthetic token (inserted by parser)
    pub fn synthetic(kind: TokenKind, span: Span) -> Self {
        let mut token = Self::new(kind, span, String::new());
        token.metadata.flags.is_synthetic = true;
        token
    }
    
    /// Create an error recovery token
    pub fn error_recovery(text: String, span: Span) -> Self {
        let mut token = Self::new(TokenKind::Invalid(text.clone()), span, text);
        token.metadata.flags.is_error_recovery = true;
        token
    }
    
    /// Get the start position of this token
    pub fn start(&self) -> usize {
        self.span.start
    }
    
    /// Get the end position of this token
    pub fn end(&self) -> usize {
        self.span.end
    }
    
    /// Get the length of this token in bytes
    pub fn len(&self) -> usize {
        self.span.end - self.span.start
    }
    
    /// Check if this token is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get the line number of this token
    pub fn line(&self) -> u32 {
        self.span.line
    }
    
    /// Get the column number of this token
    pub fn column(&self) -> u32 {
        self.span.column
    }
    
    /// Check if this token contains Unicode characters
    pub fn has_unicode(&self) -> bool {
        self.metadata.flags.has_unicode || self.text.chars().any(|c| !c.is_ascii())
    }
    
    /// Check if this token contains escape sequences
    pub fn has_escapes(&self) -> bool {
        self.metadata.flags.has_escapes || self.text.contains('\\')
    }
    
    /// Check if this token is synthetic
    pub fn is_synthetic(&self) -> bool {
        self.metadata.flags.is_synthetic
    }
    
    /// Check if this token was created during error recovery
    pub fn is_error_recovery(&self) -> bool {
        self.metadata.flags.is_error_recovery
    }
    
    /// Check if this token should be ignored in semantic analysis
    pub fn is_ignored(&self) -> bool {
        self.metadata.flags.is_ignored
    }
    
    /// Check if this token is trivia (whitespace, comments)
    pub fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }
    
    /// Check if this token can start an expression
    pub fn can_start_expression(&self) -> bool {
        self.kind.can_start_expression()
    }
    
    /// Check if this token is a statement terminator
    pub fn is_statement_terminator(&self) -> bool {
        self.kind.is_statement_terminator()
    }
    
    /// Add leading trivia to this token
    pub fn add_leading_trivia(&mut self, trivia: TriviaToken) {
        self.metadata.leading_trivia.push(trivia);
    }
    
    /// Add trailing trivia to this token
    pub fn add_trailing_trivia(&mut self, trivia: TriviaToken) {
        self.metadata.trailing_trivia.push(trivia);
    }
    
    /// Get all trivia associated with this token
    pub fn all_trivia(&self) -> impl Iterator<Item = &TriviaToken> {
        self.metadata.leading_trivia.iter().chain(self.metadata.trailing_trivia.iter())
    }
    
    /// Get the full span including all trivia
    pub fn full_span(&self) -> Span {
        let start = self.metadata.leading_trivia
            .first()
            .map(|t| t.span.start)
            .unwrap_or(self.span.start);
            
        let end = self.metadata.trailing_trivia
            .last()
            .map(|t| t.span.end)
            .unwrap_or(self.span.end);
            
        Span::new(start, end, self.span.line, self.span.column)
    }
    
    /// Get the text including all trivia
    pub fn full_text(&self) -> String {
        let mut result = String::new();
        
        for trivia in &self.metadata.leading_trivia {
            result.push_str(&trivia.text);
        }
        
        result.push_str(&self.text);
        
        for trivia in &self.metadata.trailing_trivia {
            result.push_str(&trivia.text);
        }
        
        result
    }
    
    /// Create a copy of this token with updated metadata
    pub fn with_updated_metadata<F>(&self, updater: F) -> Self 
    where
        F: FnOnce(&mut TokenMetadata),
    {
        let mut token = self.clone();
        updater(&mut token.metadata);
        token
    }
    
    /// Get syntax highlighting class for this token
    pub fn syntax_class(&self) -> &'static str {
        self.kind.syntax_class()
    }
    
    /// Check if this token represents a specific keyword
    pub fn is_keyword(&self, keyword: &str) -> bool {
        match &self.kind {
            TokenKind::Keyword(k) => k == keyword,
            _ => false,
        }
    }
    
    /// Check if this token represents a specific operator
    pub fn is_operator(&self, op: &OperatorToken) -> bool {
        match &self.kind {
            TokenKind::Operator(o) => o == op,
            _ => false,
        }
    }
    
    /// Get the identifier name if this is an identifier token
    pub fn identifier_name(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Identifier(name) => Some(name),
            TokenKind::Keyword(name) => Some(name),
            TokenKind::ContextualKeyword(name) => Some(name),
            _ => None,
        }
    }
    
    /// Get the numeric value if this is a number token
    pub fn numeric_value(&self) -> Option<f64> {
        match &self.kind {
            TokenKind::Number(NumberLiteral::Number(n)) => Some(*n),
            _ => None,
        }
    }
    
    /// Get the string value if this is a string token
    pub fn string_value(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::String(StringLiteral::String(s)) => Some(s),
            _ => None,
        }
    }
}

impl TriviaToken {
    /// Create a new trivia token
    pub fn new(kind: TriviaKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }
    
    /// Create whitespace trivia
    pub fn whitespace(text: String, span: Span) -> Self {
        Self::new(TriviaKind::Whitespace, span, text)
    }
    
    /// Create line terminator trivia
    pub fn line_terminator(text: String, span: Span) -> Self {
        Self::new(TriviaKind::LineTerminator, span, text)
    }
    
    /// Create line comment trivia
    pub fn line_comment(text: String, span: Span) -> Self {
        Self::new(TriviaKind::LineComment, span, text)
    }
    
    /// Create block comment trivia
    pub fn block_comment(text: String, span: Span) -> Self {
        Self::new(TriviaKind::BlockComment, span, text)
    }
    
    /// Check if this trivia is whitespace
    pub fn is_whitespace(&self) -> bool {
        matches!(self.kind, TriviaKind::Whitespace)
    }
    
    /// Check if this trivia is a line terminator
    pub fn is_line_terminator(&self) -> bool {
        matches!(self.kind, TriviaKind::LineTerminator)
    }
    
    /// Check if this trivia is a comment
    pub fn is_comment(&self) -> bool {
        matches!(
            self.kind,
            TriviaKind::LineComment | TriviaKind::BlockComment | TriviaKind::HtmlComment
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for TriviaToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

/// Builder for creating tokens with complex metadata
#[derive(Debug, Default)]
pub struct TokenBuilder {
    kind: Option<TokenKind>,
    span: Option<Span>,
    text: Option<String>,
    metadata: TokenMetadata,
}

impl TokenBuilder {
    /// Create a new token builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the token kind
    pub fn kind(mut self, kind: TokenKind) -> Self {
        self.kind = Some(kind);
        self
    }
    
    /// Set the token span
    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    
    /// Set the token text
    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }
    
    /// Set leading whitespace flag
    pub fn leading_whitespace(mut self, has: bool) -> Self {
        self.metadata.has_leading_whitespace = has;
        self
    }
    
    /// Set trailing whitespace flag
    pub fn trailing_whitespace(mut self, has: bool) -> Self {
        self.metadata.has_trailing_whitespace = has;
        self
    }
    
    /// Set new line flag
    pub fn new_line(mut self, is: bool) -> Self {
        self.metadata.is_new_line = is;
        self
    }
    
    /// Add leading trivia
    pub fn leading_trivia(mut self, trivia: TriviaToken) -> Self {
        self.metadata.leading_trivia.push(trivia);
        self
    }
    
    /// Add trailing trivia
    pub fn trailing_trivia(mut self, trivia: TriviaToken) -> Self {
        self.metadata.trailing_trivia.push(trivia);
        self
    }
    
    /// Set token flags
    pub fn flags(mut self, flags: TokenFlags) -> Self {
        self.metadata.flags = flags;
        self
    }
    
    /// Build the token
    pub fn build(self) -> Result<Token, &'static str> {
        let kind = self.kind.ok_or("Token kind is required")?;
        let span = self.span.ok_or("Token span is required")?;
        let text = self.text.unwrap_or_default();
        
        Ok(Token::with_metadata(kind, span, text, self.metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{NumberLiteral, OperatorToken};
    
    #[test]
    fn test_token_creation() {
        let span = Span::new(0, 5, 1, 1);
        let token = Token::new(
            TokenKind::Identifier("hello".to_string()),
            span,
            "hello".to_string(),
        );
        
        assert_eq!(token.start(), 0);
        assert_eq!(token.end(), 5);
        assert_eq!(token.len(), 5);
        assert_eq!(token.line(), 1);
        assert_eq!(token.column(), 1);
        assert_eq!(token.text, "hello");
    }
    
    #[test]
    fn test_synthetic_token() {
        let span = Span::new(0, 0, 1, 1);
        let token = Token::synthetic(
            TokenKind::Operator(OperatorToken::Semicolon),
            span,
        );
        
        assert!(token.is_synthetic());
        assert!(!token.is_error_recovery());
    }
    
    #[test]
    fn test_error_recovery_token() {
        let span = Span::new(0, 3, 1, 1);
        let token = Token::error_recovery("bad".to_string(), span);
        
        assert!(token.is_error_recovery());
        assert!(!token.is_synthetic());
        assert!(matches!(token.kind, TokenKind::Invalid(_)));
    }
    
    #[test]
    fn test_trivia_management() {
        let span = Span::new(0, 5, 1, 1);
        let mut token = Token::new(
            TokenKind::Identifier("hello".to_string()),
            span,
            "hello".to_string(),
        );
        
        let leading = TriviaToken::whitespace("  ".to_string(), Span::new(0, 2, 1, 1));
        let trailing = TriviaToken::line_comment("// comment".to_string(), Span::new(5, 15, 1, 6));
        
        token.add_leading_trivia(leading);
        token.add_trailing_trivia(trailing);
        
        assert_eq!(token.metadata.leading_trivia.len(), 1);
        assert_eq!(token.metadata.trailing_trivia.len(), 1);
        assert_eq!(token.all_trivia().count(), 2);
    }
    
    #[test]
    fn test_full_span_and_text() {
        let span = Span::new(2, 7, 1, 3);
        let mut token = Token::new(
            TokenKind::Identifier("hello".to_string()),
            span,
            "hello".to_string(),
        );
        
        let leading = TriviaToken::whitespace("  ".to_string(), Span::new(0, 2, 1, 1));
        let trailing = TriviaToken::line_comment(" // comment".to_string(), Span::new(7, 18, 1, 8));
        
        token.add_leading_trivia(leading);
        token.add_trailing_trivia(trailing);
        
        let full_span = token.full_span();
        assert_eq!(full_span.start, 0);
        assert_eq!(full_span.end, 18);
        
        let full_text = token.full_text();
        assert_eq!(full_text, "  hello // comment");
    }
    
    #[test]
    fn test_token_classification() {
        let number_token = Token::new(
            TokenKind::Number(NumberLiteral::Number(42.0)),
            Span::new(0, 2, 1, 1),
            "42".to_string(),
        );
        
        assert!(!number_token.is_trivia());
        assert!(number_token.can_start_expression());
        assert!(!number_token.is_statement_terminator());
        assert_eq!(number_token.syntax_class(), "number");
        assert_eq!(number_token.numeric_value(), Some(42.0));
    }
    
    #[test]
    fn test_token_builder() {
        let span = Span::new(0, 5, 1, 1);
        let token = TokenBuilder::new()
            .kind(TokenKind::Identifier("hello".to_string()))
            .span(span)
            .text("hello".to_string())
            .leading_whitespace(true)
            .new_line(false)
            .build()
            .unwrap();
        
        assert_eq!(token.text, "hello");
        assert!(token.metadata.has_leading_whitespace);
        assert!(!token.metadata.is_new_line);
    }
    
    #[test]
    fn test_trivia_token_classification() {
        let whitespace = TriviaToken::whitespace("  ".to_string(), Span::new(0, 2, 1, 1));
        let line_term = TriviaToken::line_terminator("\n".to_string(), Span::new(2, 3, 1, 3));
        let comment = TriviaToken::line_comment("// hello".to_string(), Span::new(3, 11, 2, 1));
        
        assert!(whitespace.is_whitespace());
        assert!(!whitespace.is_comment());
        
        assert!(line_term.is_line_terminator());
        assert!(!line_term.is_comment());
        
        assert!(comment.is_comment());
        assert!(!comment.is_whitespace());
    }
}
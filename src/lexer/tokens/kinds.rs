//! Enhanced token kinds for JavaScript lexer
//!
//! Provides a comprehensive token classification system that unifies
//! tokens from all the specialized scanners.

use crate::lexer::{
    NumberLiteral,
    StringLiteral, 
    IdentifierToken,
    OperatorToken,
    CommentToken,
};

/// Comprehensive token kind enum that encompasses all JavaScript tokens
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    /// Number literal (42, 3.14, 0xFF, 123n)
    Number(NumberLiteral),
    /// String literal ("hello", 'world')
    String(StringLiteral),
    /// Boolean literal (true, false)
    Boolean(bool),
    /// Null literal
    Null,
    /// Undefined literal
    Undefined,
    
    // Identifiers and keywords
    /// Regular identifier (variable names, function names, etc.)
    Identifier(String),
    /// Reserved keyword (function, var, if, etc.)
    Keyword(String),
    /// Contextual keyword (async, await, from, of)
    ContextualKeyword(String),
    
    // Operators and punctuation
    /// Operator or punctuation token
    Operator(OperatorToken),
    
    // Comments (usually skipped but can be preserved)
    /// Comment token
    Comment(CommentToken),
    
    // Whitespace and formatting
    /// Whitespace (spaces, tabs, etc.)
    Whitespace(String),
    /// Line terminator (\n, \r\n, etc.)
    LineTerminator(String),
    
    // Template literals (for future expansion)
    /// Template literal start (`hello ${)
    TemplateStart(String),
    /// Template literal middle (} world ${)  
    TemplateMiddle(String),
    /// Template literal end (} end`)
    TemplateEnd(String),
    /// Complete template literal without expressions (`hello world`)
    TemplateNoSubstitution(String),
    
    // Regular expressions (for future expansion)
    /// Regular expression literal (/pattern/flags)
    RegExp { pattern: String, flags: String },
    
    // Special tokens
    /// End of file
    Eof,
    /// Invalid token (error recovery)
    Invalid(String),
    
    // Legacy compatibility variants (direct mappings for common punctuation)
    /// Left brace {
    LeftBrace,
    /// Right brace }
    RightBrace,
    /// Left parenthesis (
    LeftParen,
    /// Right parenthesis )
    RightParen,
    /// Left bracket [
    LeftBracket,
    /// Right bracket ]
    RightBracket,
    /// Semicolon ;
    Semicolon,
    /// Comma ,
    Comma,
    /// Dot .
    Dot,
    /// Newline character
    Newline,
}

impl TokenKind {
    /// Check if this token is a literal value
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::Number(_) |
            TokenKind::String(_) |
            TokenKind::Boolean(_) |
            TokenKind::Null |
            TokenKind::Undefined |
            TokenKind::RegExp { .. } |
            TokenKind::TemplateNoSubstitution(_)
        )
    }
    
    /// Check if this token is an identifier (including keywords)
    pub fn is_identifier(&self) -> bool {
        matches!(
            self,
            TokenKind::Identifier(_) |
            TokenKind::Keyword(_) |
            TokenKind::ContextualKeyword(_)
        )
    }
    
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(self, TokenKind::Keyword(_))
    }
    
    /// Check if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(self, TokenKind::Operator(_))
    }
    
    /// Check if this token is punctuation
    pub fn is_punctuation(&self) -> bool {
        if let TokenKind::Operator(op) = self {
            use crate::lexer::scanners::operators::OperatorToken;
            matches!(
                op,
                OperatorToken::LeftParen | OperatorToken::RightParen |
                OperatorToken::LeftBrace | OperatorToken::RightBrace |
                OperatorToken::LeftBracket | OperatorToken::RightBracket |
                OperatorToken::Semicolon | OperatorToken::Comma |
                OperatorToken::Dot | OperatorToken::Colon |
                OperatorToken::Question | OperatorToken::Arrow
            )
        } else {
            false
        }
    }
    
    /// Check if this token is whitespace or formatting
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace(_) |
            TokenKind::LineTerminator(_) |
            TokenKind::Comment(_)
        )
    }
    
    /// Check if this token can start an expression
    pub fn can_start_expression(&self) -> bool {
        matches!(
            self,
            TokenKind::Number(_) |
            TokenKind::String(_) |
            TokenKind::Boolean(_) |
            TokenKind::Null |
            TokenKind::Undefined |
            TokenKind::Identifier(_) |
            TokenKind::RegExp { .. } |
            TokenKind::TemplateStart(_) |
            TokenKind::TemplateNoSubstitution(_)
        ) || self.is_unary_operator() || self.is_left_paren()
    }
    
    /// Check if this token is a unary operator
    pub fn is_unary_operator(&self) -> bool {
        if let TokenKind::Operator(op) = self {
            use crate::lexer::scanners::operators::OperatorToken;
            matches!(
                op,
                OperatorToken::Plus | OperatorToken::Minus |
                OperatorToken::Bang | OperatorToken::Tilde |
                OperatorToken::PlusPlus | OperatorToken::MinusMinus
            )
        } else {
            false
        }
    }
    
    /// Check if this token is a binary operator
    pub fn is_binary_operator(&self) -> bool {
        if let TokenKind::Operator(op) = self {
            use crate::lexer::scanners::operators::OperatorToken;
            !matches!(
                op,
                OperatorToken::LeftParen | OperatorToken::RightParen |
                OperatorToken::LeftBrace | OperatorToken::RightBrace |
                OperatorToken::LeftBracket | OperatorToken::RightBracket |
                OperatorToken::Semicolon | OperatorToken::Comma |
                OperatorToken::Colon
            )
        } else {
            false
        }
    }
    
    /// Check if this token is an assignment operator
    pub fn is_assignment_operator(&self) -> bool {
        if let TokenKind::Operator(op) = self {
            use crate::lexer::scanners::operators::OperatorToken;
            matches!(
                op,
                OperatorToken::Equal | OperatorToken::PlusEqual |
                OperatorToken::MinusEqual | OperatorToken::StarEqual |
                OperatorToken::SlashEqual | OperatorToken::PercentEqual |
                OperatorToken::StarStarEqual | OperatorToken::AmpEqual |
                OperatorToken::PipeEqual | OperatorToken::CaretEqual |
                OperatorToken::LessLessEqual | OperatorToken::GreaterGreaterEqual |
                OperatorToken::GreaterGreaterGreaterEqual | 
                OperatorToken::QuestionQuestionEqual |
                OperatorToken::AmpAmpEqual | OperatorToken::PipePipeEqual
            )
        } else {
            false
        }
    }
    
    /// Check if this token is a left parenthesis
    pub fn is_left_paren(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::LeftParen)
        )
    }
    
    /// Check if this token is a right parenthesis
    pub fn is_right_paren(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::RightParen)
        )
    }
    
    /// Check if this token is a left brace
    pub fn is_left_brace(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::LeftBrace)
        )
    }
    
    /// Check if this token is a right brace
    pub fn is_right_brace(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::RightBrace)
        )
    }
    
    /// Check if this token is a semicolon
    pub fn is_semicolon(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::Semicolon)
        )
    }
    
    /// Check if this token is a comma
    pub fn is_comma(&self) -> bool {
        matches!(
            self,
            TokenKind::Operator(OperatorToken::Comma)
        )
    }
    
    /// Get the text representation of this token kind
    pub fn as_str(&self) -> &str {
        match self {
            TokenKind::Number(_) => "number",
            TokenKind::String(_) => "string",
            TokenKind::Boolean(_) => "boolean",
            TokenKind::Null => "null",
            TokenKind::Undefined => "undefined",
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Keyword(_) => "keyword",
            TokenKind::ContextualKeyword(_) => "contextual-keyword",
            TokenKind::Operator(_) => "operator",
            TokenKind::Comment(_) => "comment",
            TokenKind::Whitespace(_) => "whitespace",
            TokenKind::LineTerminator(_) => "line-terminator",
            TokenKind::TemplateStart(_) => "template-start",
            TokenKind::TemplateMiddle(_) => "template-middle",
            TokenKind::TemplateEnd(_) => "template-end",
            TokenKind::TemplateNoSubstitution(_) => "template-literal",
            TokenKind::RegExp { .. } => "regexp",
            TokenKind::Eof => "eof",
            TokenKind::Invalid(_) => "invalid",
            // Legacy compatibility variants
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Semicolon => ";",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Newline => "newline",
        }
    }
    
    /// Get the precedence of this token if it's an operator
    pub fn precedence(&self) -> Option<crate::lexer::scanners::operators::Precedence> {
        if let TokenKind::Operator(_op) = self {
            // This would require access to the operator scanner's precedence table
            // For now, return None - this would be implemented with a static lookup
            None
        } else {
            None
        }
    }
}

/// Convert from scanner-specific tokens to unified TokenKind
impl From<NumberLiteral> for TokenKind {
    fn from(literal: NumberLiteral) -> Self {
        TokenKind::Number(literal)
    }
}

impl From<StringLiteral> for TokenKind {
    fn from(literal: StringLiteral) -> Self {
        TokenKind::String(literal)
    }
}

impl From<IdentifierToken> for TokenKind {
    fn from(token: IdentifierToken) -> Self {
        match token {
            IdentifierToken::Identifier(name) => TokenKind::Identifier(name),
            IdentifierToken::Keyword(keyword) => TokenKind::Keyword(keyword),
            IdentifierToken::ContextualKeyword(keyword) => TokenKind::ContextualKeyword(keyword),
            IdentifierToken::Boolean(value) => TokenKind::Boolean(value),
            IdentifierToken::Null => TokenKind::Null,
            IdentifierToken::Undefined => TokenKind::Undefined,
        }
    }
}

impl From<OperatorToken> for TokenKind {
    fn from(token: OperatorToken) -> Self {
        TokenKind::Operator(token)
    }
}

impl From<CommentToken> for TokenKind {
    fn from(token: CommentToken) -> Self {
        TokenKind::Comment(token)
    }
}

/// Token classification helpers
impl TokenKind {
    /// Classify token for syntax highlighting
    pub fn syntax_class(&self) -> &'static str {
        match self {
            TokenKind::Number(_) => "number",
            TokenKind::String(_) => "string",
            TokenKind::Boolean(_) | TokenKind::Null | TokenKind::Undefined => "literal",
            TokenKind::Identifier(_) => "identifier",
            TokenKind::Keyword(_) => "keyword",
            TokenKind::ContextualKeyword(_) => "keyword-contextual",
            TokenKind::Operator(_) => "operator",
            TokenKind::Comment(_) => "comment",
            TokenKind::Whitespace(_) => "whitespace",
            TokenKind::LineTerminator(_) => "line-break",
            TokenKind::TemplateStart(_) | TokenKind::TemplateMiddle(_) | 
            TokenKind::TemplateEnd(_) | TokenKind::TemplateNoSubstitution(_) => "template",
            TokenKind::RegExp { .. } => "regexp",
            TokenKind::Eof => "eof",
            TokenKind::Invalid(_) => "error",
            // Legacy compatibility variants
            TokenKind::LeftBrace => "operator",
            TokenKind::RightBrace => "operator",
            TokenKind::LeftParen => "operator", 
            TokenKind::RightParen => "operator",
            TokenKind::LeftBracket => "operator",
            TokenKind::RightBracket => "operator",
            TokenKind::Semicolon => "operator",
            TokenKind::Comma => "operator",
            TokenKind::Dot => "operator",
            TokenKind::Newline => "line-break",
        }
    }
    
    /// Check if token should be included in semantic analysis
    pub fn is_semantic(&self) -> bool {
        !self.is_trivia() && !matches!(self, TokenKind::Invalid(_))
    }
    
    /// Check if token represents a statement terminator
    pub fn is_statement_terminator(&self) -> bool {
        self.is_semicolon() || 
        matches!(self, TokenKind::LineTerminator(_)) ||
        matches!(self, TokenKind::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{NumberLiteral, OperatorToken};
    
    #[test]
    fn test_literal_classification() {
        assert!(TokenKind::Number(NumberLiteral::Number(42.0)).is_literal());
        assert!(TokenKind::Boolean(true).is_literal());
        assert!(TokenKind::Null.is_literal());
        assert!(!TokenKind::Identifier("hello".to_string()).is_literal());
    }
    
    #[test]
    fn test_identifier_classification() {
        assert!(TokenKind::Identifier("hello".to_string()).is_identifier());
        assert!(TokenKind::Keyword("function".to_string()).is_identifier());
        assert!(TokenKind::ContextualKeyword("async".to_string()).is_identifier());
        assert!(!TokenKind::Number(NumberLiteral::Number(42.0)).is_identifier());
    }
    
    #[test]
    fn test_operator_classification() {
        assert!(TokenKind::Operator(OperatorToken::Plus).is_operator());
        assert!(TokenKind::Operator(OperatorToken::Equal).is_assignment_operator());
        assert!(TokenKind::Operator(OperatorToken::Bang).is_unary_operator());
        assert!(!TokenKind::Identifier("hello".to_string()).is_operator());
    }
    
    #[test]
    fn test_punctuation_classification() {
        assert!(TokenKind::Operator(OperatorToken::LeftParen).is_punctuation());
        assert!(TokenKind::Operator(OperatorToken::Semicolon).is_punctuation());
        assert!(!TokenKind::Operator(OperatorToken::Plus).is_punctuation());
    }
    
    #[test]
    fn test_trivia_classification() {
        assert!(TokenKind::Whitespace(" ".to_string()).is_trivia());
        assert!(TokenKind::LineTerminator("\n".to_string()).is_trivia());
        assert!(!TokenKind::Identifier("hello".to_string()).is_trivia());
    }
    
    #[test]
    fn test_expression_start() {
        assert!(TokenKind::Number(NumberLiteral::Number(42.0)).can_start_expression());
        assert!(TokenKind::Identifier("hello".to_string()).can_start_expression());
        assert!(TokenKind::Operator(OperatorToken::LeftParen).can_start_expression());
        assert!(!TokenKind::Operator(OperatorToken::RightParen).can_start_expression());
    }
    
    #[test]
    fn test_token_conversion() {
        let number = NumberLiteral::Number(42.0);
        let token = TokenKind::from(number);
        assert!(matches!(token, TokenKind::Number(_)));
        
        let op = OperatorToken::Plus;
        let token = TokenKind::from(op);
        assert!(matches!(token, TokenKind::Operator(_)));
    }
    
    #[test]
    fn test_syntax_classification() {
        assert_eq!(TokenKind::Number(NumberLiteral::Number(42.0)).syntax_class(), "number");
        assert_eq!(TokenKind::Keyword("function".to_string()).syntax_class(), "keyword");
        assert_eq!(TokenKind::Operator(OperatorToken::Plus).syntax_class(), "operator");
    }
    
    #[test]
    fn test_semantic_classification() {
        assert!(TokenKind::Identifier("hello".to_string()).is_semantic());
        assert!(!TokenKind::Whitespace(" ".to_string()).is_semantic());
        assert!(!TokenKind::Invalid("bad".to_string()).is_semantic());
    }
    
    #[test]
    fn test_statement_terminators() {
        assert!(TokenKind::Operator(OperatorToken::Semicolon).is_statement_terminator());
        assert!(TokenKind::LineTerminator("\n".to_string()).is_statement_terminator());
        assert!(TokenKind::Eof.is_statement_terminator());
        assert!(!TokenKind::Identifier("hello".to_string()).is_statement_terminator());
    }
}
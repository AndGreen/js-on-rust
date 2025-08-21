//! Parser core utilities and token management
//! 
//! Provides shared token management functionality and coordination utilities
//! for the specialized parsing agents.

use super::ast::*;
use crate::error::{Error, Result, Span};
use crate::lexer::{Token, TokenKind};

/// Operator precedence for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,  // =
    Or = 2,         // ||
    And = 3,        // &&
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Term = 6,       // + -
    Factor = 7,     // * /
    Unary = 8,      // ! -
    Call = 9,       // . ()
    Primary = 10,
}

impl Precedence {
    pub fn next(self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

/// Core parser functionality for token management and coordination
pub struct ParserCore {
    pub tokens: Vec<Token>,
    pub current: usize,
}

impl ParserCore {
    /// Create a new parser core from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }
    
    /// Check if we're at the end of tokens
    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.peek().kind, TokenKind::Eof)
    }
    
    /// Get the current token without consuming it
    pub fn peek(&self) -> &Token {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            // If we're at the end, return the last token (should be EOF)
            &self.tokens[self.tokens.len() - 1]
        }
    }
    
    /// Get the previous token
    pub fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }
    
    /// Advance to the next token and return the current one
    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    /// Check if current token matches and consume if it does
    pub fn match_token(&mut self, token_type: &TokenKind) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    /// Check if current token is of given type
    pub fn check(&self, token_type: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(token_type)
        }
    }
    
    /// Consume token of expected type or error
    pub fn consume(&mut self, token_type: &TokenKind, message: &str) -> Result<()> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(Error::parser(
                message.to_string(),
                self.peek().span,
            ))
        }
    }
    
    /// Consume identifier token
    pub fn consume_identifier(&mut self, message: &str) -> Result<String> {
        if let TokenKind::Identifier(name) = &self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(Error::parser(
                message.to_string(),
                self.peek().span,
            ))
        }
    }
    
    /// Consume semicolon or newline (both are valid statement terminators)
    pub fn consume_semicolon_or_newline(&mut self) {
        if matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof | TokenKind::RightBrace) {
            // Only consume if it's not a closing brace (which ends the block)
            if !matches!(self.peek().kind, TokenKind::RightBrace) {
                self.advance();
            }
        }
    }
    
    /// Get precedence for a token
    pub fn get_precedence(&self, token: &TokenKind) -> Precedence {
        match token {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual |
            TokenKind::EqualEqualEqual | TokenKind::BangEqualEqual => Precedence::Equality,
            TokenKind::Less | TokenKind::Greater |
            TokenKind::LessEqual | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            _ => Precedence::None,
        }
    }
    
    /// Convert token to binary operator
    pub fn token_to_binary_op(&self, token: &TokenKind) -> Result<BinaryOp> {
        match token {
            TokenKind::Plus => Ok(BinaryOp::Add),
            TokenKind::Minus => Ok(BinaryOp::Subtract),
            TokenKind::Star => Ok(BinaryOp::Multiply),
            TokenKind::Slash => Ok(BinaryOp::Divide),
            TokenKind::Percent => Ok(BinaryOp::Modulo),
            TokenKind::EqualEqual => Ok(BinaryOp::Equal),
            TokenKind::BangEqual => Ok(BinaryOp::NotEqual),
            TokenKind::EqualEqualEqual => Ok(BinaryOp::StrictEqual),
            TokenKind::BangEqualEqual => Ok(BinaryOp::StrictNotEqual),
            TokenKind::Less => Ok(BinaryOp::Less),
            TokenKind::Greater => Ok(BinaryOp::Greater),
            TokenKind::LessEqual => Ok(BinaryOp::LessEqual),
            TokenKind::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            TokenKind::AmpAmp => Ok(BinaryOp::LogicalAnd),
            TokenKind::PipePipe => Ok(BinaryOp::LogicalOr),
            _ => Err(Error::parser(
                format!("Invalid binary operator: {}", token),
                Span::new(0, 0, 1, 1), // TODO: use actual span
            )),
        }
    }
    
    /// Get the binary operator for compound assignment
    pub fn get_compound_assignment_op(&self, token: &TokenKind) -> Option<BinaryOp> {
        match token {
            TokenKind::PlusEqual => Some(BinaryOp::Add),
            TokenKind::MinusEqual => Some(BinaryOp::Subtract),
            TokenKind::StarEqual => Some(BinaryOp::Multiply),
            TokenKind::SlashEqual => Some(BinaryOp::Divide),
            TokenKind::PercentEqual => Some(BinaryOp::Modulo),
            _ => None,
        }
    }
}
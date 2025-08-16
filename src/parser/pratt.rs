//! Pratt parser for JavaScript expressions
//! 
//! Implements operator precedence parsing using the Pratt parsing technique.

use super::ast::*;
use crate::error::Result;
use crate::lexer::{Token, TokenKind};

/// Operator precedence levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,  // =
    Or = 2,          // ||
    And = 3,         // &&
    Equality = 4,    // == !=
    Comparison = 5,  // < > <= >=
    Term = 6,        // + -
    Factor = 7,      // * /
    Unary = 8,       // ! -
    Call = 9,        // . ()
    Primary = 10,
}

/// Pratt parser for expressions
pub struct PrattParser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> PrattParser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }
    
    /// Parse an expression with the given precedence
    pub fn parse_expression(&mut self, _precedence: Precedence) -> Result<Expr> {
        // TODO: Implement Pratt parsing
        // For now, return a dummy expression
        Ok(Expr::Literal(Literal::Number(0.0)))
    }
    
    /// Get the precedence for a token type
    fn get_precedence(&self, token_kind: &TokenKind) -> Precedence {
        match token_kind {
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash => Precedence::Factor,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less | TokenKind::Greater => Precedence::Comparison,
            _ => Precedence::None,
        }
    }
}
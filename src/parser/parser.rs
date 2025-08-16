//! JavaScript parser implementation
//! 
//! Uses recursive descent parsing with Pratt parser for expressions.

use super::ast::*;
use crate::error::{Error, Result, Span};
use crate::lexer::{Token, TokenKind};

/// Create EOF token
fn make_eof_token() -> Token {
    Token {
        kind: TokenKind::Eof,
        span: Span::new(0, 0, 1, 1),
        text: String::new(),
    }
}

/// JavaScript parser
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }
    
    /// Parse the tokens into an AST
    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip EOF token
            if matches!(self.peek().kind, TokenKind::Eof) {
                break;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        
        Ok(Program { statements })
    }
    
    /// Parse a statement (placeholder implementation)
    fn parse_statement(&mut self) -> Result<Stmt> {
        // TODO: Implement proper statement parsing
        // For now, just consume one token and create a dummy expression statement
        
        if self.is_at_end() {
            return Err(Error::parser(
                "Unexpected end of input".to_string(),
                self.previous().span,
            ));
        }
        
        let token = self.advance();
        let span = token.span;
        
        // Create a dummy literal expression
        let expr = Expr::Literal(Literal::Number(42.0));
        
        Ok(Stmt::Expression(expr))
    }
    
    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.peek().kind, TokenKind::Eof)
    }
    
    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        if self.current < self.tokens.len() {
            &self.tokens[self.current]
        } else {
            // If we're at the end, return the last token (should be EOF)
            &self.tokens[self.tokens.len() - 1]
        }
    }
    
    /// Get the previous token
    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }
    
    /// Advance to the next token and return the current one
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
}
//! JavaScript parser implementation
//! 
//! Uses recursive descent parsing with Pratt parser for expressions.

use super::ast::*;
use crate::error::{Error, Result, Span};
use crate::lexer::{Token, TokenKind, Keyword};

/// Operator precedence for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
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
            // Skip newlines
            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
                continue;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        
        Ok(Program { statements })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Stmt> {
        match &self.peek().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let_declaration(),
            TokenKind::Keyword(Keyword::Var) => self.parse_var_declaration(),
            TokenKind::Keyword(Keyword::Const) => self.parse_const_declaration(),
            _ => self.parse_expression_statement(),
        }
    }
    
    /// Parse let declaration: `let x = expr`
    fn parse_let_declaration(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'let'
        
        let name = self.consume_identifier("Expected variable name")?;
        
        let init = if self.match_token(&TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse var declaration: `var x = expr`
    fn parse_var_declaration(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'var'
        
        let name = self.consume_identifier("Expected variable name")?;
        
        let init = if self.match_token(&TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse const declaration: `const x = expr`
    fn parse_const_declaration(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'const'
        
        let name = self.consume_identifier("Expected variable name")?;
        
        if !self.match_token(&TokenKind::Equal) {
            return Err(Error::parser(
                "Missing initializer in const declaration".to_string(),
                self.peek().span,
            ));
        }
        
        let init = Some(self.parse_expression()?);
        
        self.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse expression statement: `expr;`
    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        self.consume_semicolon_or_newline();
        Ok(Stmt::Expression(expr))
    }
    
    /// Parse expression using Pratt parsing
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_precedence(Precedence::Assignment)
    }
    
    /// Parse expression with given minimum precedence
    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        
        while !self.is_at_end() && precedence <= self.get_precedence(&self.peek().kind) {
            left = self.parse_binary(left)?;
        }
        
        Ok(left)
    }
    
    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expr> {
        match &self.peek().kind {
            TokenKind::Minus => {
                let span = self.peek().span;
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    operand,
                    span,
                })
            }
            TokenKind::Plus => {
                let span = self.peek().span;
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    operand,
                    span,
                })
            }
            TokenKind::Bang => {
                let span = self.peek().span;
                self.advance();
                let operand = Box::new(self.parse_unary()?);
                Ok(Expr::Unary {
                    op: UnaryOp::LogicalNot,
                    operand,
                    span,
                })
            }
            _ => self.parse_primary(),
        }
    }
    
    /// Parse binary expression
    fn parse_binary(&mut self, left: Expr) -> Result<Expr> {
        let op_token = self.advance().clone();
        let precedence = self.get_precedence(&op_token.kind);
        let right = self.parse_precedence(precedence.next())?;
        
        let op = self.token_to_binary_op(&op_token.kind)?;
        
        Ok(Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: op_token.span,
        })
    }
    
    /// Parse primary expression (literals, identifiers, parenthesized expressions)
    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.advance();
        
        match &token.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(Literal::Number(*n))),
            TokenKind::String(s) => Ok(Expr::Literal(Literal::String(s.clone()))),
            TokenKind::Boolean(b) => Ok(Expr::Literal(Literal::Boolean(*b))),
            TokenKind::Null => Ok(Expr::Literal(Literal::Null)),
            TokenKind::Undefined => Ok(Expr::Literal(Literal::Undefined)),
            TokenKind::Keyword(Keyword::True) => Ok(Expr::Literal(Literal::Boolean(true))),
            TokenKind::Keyword(Keyword::False) => Ok(Expr::Literal(Literal::Boolean(false))),
            TokenKind::Keyword(Keyword::Null) => Ok(Expr::Literal(Literal::Null)),
            TokenKind::Keyword(Keyword::Undefined) => Ok(Expr::Literal(Literal::Undefined)),
            TokenKind::Identifier(name) => Ok(Expr::Identifier {
                name: name.clone(),
                span: token.span,
            }),
            TokenKind::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(&TokenKind::RightParen, "Expected ')'")?;
                Ok(expr)
            }
            _ => Err(Error::parser(
                format!("Unexpected token: {}", token.kind),
                token.span,
            )),
        }
    }
    
    /// Get precedence for a token
    fn get_precedence(&self, token: &TokenKind) -> Precedence {
        match token {
            TokenKind::Equal => Precedence::Assignment,
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
    fn token_to_binary_op(&self, token: &TokenKind) -> Result<BinaryOp> {
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
    
    /// Helper methods for token consumption
    
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
    
    /// Check if current token matches and consume if it does
    fn match_token(&mut self, token_type: &TokenKind) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    /// Check if current token is of given type
    fn check(&self, token_type: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(token_type)
        }
    }
    
    /// Consume token of expected type or error
    fn consume(&mut self, token_type: &TokenKind, message: &str) -> Result<()> {
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
    fn consume_identifier(&mut self, message: &str) -> Result<String> {
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
    fn consume_semicolon_or_newline(&mut self) {
        if matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof) {
            self.advance();
        }
    }
}

impl Precedence {
    fn next(self) -> Self {
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
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
            TokenKind::Keyword(Keyword::Function) => self.parse_function_declaration(),
            TokenKind::Keyword(Keyword::If) => self.parse_if_statement(),
            TokenKind::Keyword(Keyword::While) => self.parse_while_statement(),
            TokenKind::Keyword(Keyword::For) => self.parse_for_statement(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_statement(),
            TokenKind::LeftBrace => self.parse_block_statement(),
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
    
    /// Parse function declaration: `function name(params) { body }`
    fn parse_function_declaration(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'function'
        
        let name = self.consume_identifier("Expected function name")?;
        
        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        
        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            params.push(self.consume_identifier("Expected parameter name")?);
            if !self.check(&TokenKind::RightParen) {
                self.consume(&TokenKind::Comma, "Expected ',' between parameters")?;
            }
        }
        
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;
        
        self.consume(&TokenKind::LeftBrace, "Expected '{' to start function body")?;
        let body = self.parse_block_statement_body()?;
        
        Ok(Stmt::FunctionDecl { name, params, body, span: start_span })
    }
    
    /// Parse if statement: `if (test) then_stmt else else_stmt`
    fn parse_if_statement(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'if'
        
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let test = self.parse_expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after if condition")?;
        
        let then_stmt = Box::new(self.parse_statement()?);
        
        let else_stmt = if self.match_token(&TokenKind::Keyword(Keyword::Else)) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        
        Ok(Stmt::If { test, then_stmt, else_stmt, span: start_span })
    }
    
    /// Parse while statement: `while (test) body`
    fn parse_while_statement(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'while'
        
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let test = self.parse_expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after while condition")?;
        
        let body = Box::new(self.parse_statement()?);
        
        Ok(Stmt::While { test, body, span: start_span })
    }
    
    /// Parse for statement: `for (init; test; update) body`
    fn parse_for_statement(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'for'
        
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'for'")?;
        
        // Parse init (can be a variable declaration or expression, or empty)
        let init = if self.match_token(&TokenKind::Semicolon) {
            None // Empty init
        } else if matches!(self.peek().kind, TokenKind::Keyword(Keyword::Var) | TokenKind::Keyword(Keyword::Let)) {
            // Variable declaration
            Some(Box::new(self.parse_statement()?))
        } else {
            // Expression statement
            let expr = self.parse_expression()?;
            self.consume(&TokenKind::Semicolon, "Expected ';' after for loop initializer")?;
            Some(Box::new(Stmt::Expression(expr)))
        };
        
        // Parse test condition (optional)
        let test = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenKind::Semicolon, "Expected ';' after for loop condition")?;
        
        // Parse update expression (optional)
        let update = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.consume(&TokenKind::RightParen, "Expected ')' after for loop clauses")?;
        
        // Parse body
        let body = Box::new(self.parse_statement()?);
        
        Ok(Stmt::For { init, test, update, body, span: start_span })
    }
    
    /// Parse return statement: `return expr?;`
    fn parse_return_statement(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.advance(); // consume 'return'
        
        let value = if matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof | TokenKind::RightBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        self.consume_semicolon_or_newline();
        
        Ok(Stmt::Return { value, span: start_span })
    }
    
    /// Parse block statement: `{ statements }`
    fn parse_block_statement(&mut self) -> Result<Stmt> {
        let start_span = self.peek().span;
        self.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        
        let statements = self.parse_block_statement_body()?;
        
        Ok(Stmt::Block { statements, span: start_span })
    }
    
    /// Parse the body of a block (statements between braces)
    fn parse_block_statement_body(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            // Skip newlines and whitespace
            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
                continue;
            }
            
            // If we hit a closing brace, break out of the loop
            if self.check(&TokenKind::RightBrace) {
                break;
            }
            
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&TokenKind::RightBrace, "Expected '}'")?;
        Ok(statements)
    }
    
    /// Parse expression statement: `expr;`
    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_expression()?;
        self.consume_semicolon_or_newline();
        Ok(Stmt::Expression(expr))
    }
    
    /// Parse expression using Pratt parsing
    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }
    
    /// Parse assignment expressions (right-associative)
    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_precedence(Precedence::Or)?;
        
        if self.match_token(&TokenKind::Equal) {
            let start_span = self.previous().span;
            let right = self.parse_assignment()?; // Right associative
            
            return Ok(Expr::Assignment {
                left: Box::new(expr),
                right: Box::new(right),
                span: start_span,
            });
        }
        
        // Handle compound assignment operators
        if let Some(compound_op) = self.get_compound_assignment_op(&self.peek().kind) {
            let op_token = self.advance().clone();
            let right = self.parse_assignment()?; // Right associative
            
            // Transform a += b into a = a + b
            let binary_expr = Expr::Binary {
                op: compound_op,
                left: Box::new(expr.clone()),
                right: Box::new(right),
                span: op_token.span,
            };
            
            return Ok(Expr::Assignment {
                left: Box::new(expr),
                right: Box::new(binary_expr),
                span: op_token.span,
            });
        }
        
        Ok(expr)
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
        
        let mut expr = match &token.kind {
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
        }?;
        
        // Handle postfix expressions (calls, member access)
        expr = self.parse_postfix(expr)?;
        
        Ok(expr)
    }
    
    /// Parse postfix expressions (calls, member access)
    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr> {
        while !self.is_at_end() {
            match &self.peek().kind {
                TokenKind::LeftParen => {
                    // Function call: func(args)
                    let start_span = self.peek().span;
                    self.advance(); // consume '('
                    
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenKind::RightParen) {
                            self.consume(&TokenKind::Comma, "Expected ',' between arguments")?;
                        }
                    }
                    
                    self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span: start_span,
                    };
                }
                TokenKind::Dot => {
                    // Member access: obj.prop
                    let start_span = self.peek().span;
                    self.advance(); // consume '.'
                    let property_name = self.consume_identifier("Expected property name after '.'")?;
                    
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(Expr::Identifier { 
                            name: property_name, 
                            span: start_span 
                        }),
                        computed: false,
                        span: start_span,
                    };
                }
                TokenKind::LeftBracket => {
                    // Computed member access: obj[key]
                    let start_span = self.peek().span;
                    self.advance(); // consume '['
                    let property = self.parse_expression()?;
                    self.consume(&TokenKind::RightBracket, "Expected ']'")?;
                    
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                        span: start_span,
                    };
                }
                TokenKind::PlusPlus => {
                    // Postfix increment: expr++
                    let span = self.peek().span;
                    self.advance(); // consume '++'
                    
                    expr = Expr::PostfixUnary {
                        op: PostfixUnaryOp::Increment,
                        operand: Box::new(expr),
                        span,
                    };
                }
                TokenKind::MinusMinus => {
                    // Postfix decrement: expr--
                    let span = self.peek().span;
                    self.advance(); // consume '--'
                    
                    expr = Expr::PostfixUnary {
                        op: PostfixUnaryOp::Decrement,
                        operand: Box::new(expr),
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
    
    /// Get precedence for a token
    fn get_precedence(&self, token: &TokenKind) -> Precedence {
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
    
    /// Get the binary operator for compound assignment
    fn get_compound_assignment_op(&self, token: &TokenKind) -> Option<BinaryOp> {
        match token {
            TokenKind::PlusEqual => Some(BinaryOp::Add),
            TokenKind::MinusEqual => Some(BinaryOp::Subtract),
            TokenKind::StarEqual => Some(BinaryOp::Multiply),
            TokenKind::SlashEqual => Some(BinaryOp::Divide),
            TokenKind::PercentEqual => Some(BinaryOp::Modulo),
            _ => None,
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
        if matches!(self.peek().kind, TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof | TokenKind::RightBrace) {
            // Only consume if it's not a closing brace (which ends the block)
            if !matches!(self.peek().kind, TokenKind::RightBrace) {
                self.advance();
            }
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
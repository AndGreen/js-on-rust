//! Expression parsing functionality
//! 
//! Handles parsing of all JavaScript expressions using Pratt parsing
//! for proper operator precedence and associativity.

use super::ast::*;
use super::core::{ParserCore, Precedence};
use super::statements::ExpressionParser as ExpressionParserTrait;
use crate::error::{Error, Result};
use crate::lexer::{TokenKind, Keyword};

/// Expression parser for handling all expression types
pub struct ExpressionParser;

impl ExpressionParser {
    /// Create a new expression parser
    pub fn new() -> Self {
        Self
    }
    
    /// Parse assignment expressions (right-associative)
    fn parse_assignment(&mut self, core: &mut ParserCore) -> Result<Expr> {
        let expr = self.parse_precedence(core, Precedence::Or)?;
        
        if core.match_token(&TokenKind::Equal) {
            let start_span = core.previous().span;
            let right = self.parse_assignment(core)?; // Right associative
            
            return Ok(Expr::Assignment {
                left: Box::new(expr),
                right: Box::new(right),
                span: start_span,
            });
        }
        
        // Handle compound assignment operators
        if let Some(compound_op) = core.get_compound_assignment_op(&core.peek().kind) {
            let op_token = core.advance().clone();
            let right = self.parse_assignment(core)?; // Right associative
            
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
    fn parse_precedence(&mut self, core: &mut ParserCore, precedence: Precedence) -> Result<Expr> {
        let mut left = self.parse_unary(core)?;
        
        while !core.is_at_end() && precedence <= core.get_precedence(&core.peek().kind) {
            left = self.parse_binary(core, left)?;
        }
        
        Ok(left)
    }
    
    /// Parse unary expression
    fn parse_unary(&mut self, core: &mut ParserCore) -> Result<Expr> {
        match &core.peek().kind {
            TokenKind::Minus => {
                let span = core.peek().span;
                core.advance();
                let operand = Box::new(self.parse_unary(core)?);
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    operand,
                    span,
                })
            }
            TokenKind::Plus => {
                let span = core.peek().span;
                core.advance();
                let operand = Box::new(self.parse_unary(core)?);
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    operand,
                    span,
                })
            }
            TokenKind::Bang => {
                let span = core.peek().span;
                core.advance();
                let operand = Box::new(self.parse_unary(core)?);
                Ok(Expr::Unary {
                    op: UnaryOp::LogicalNot,
                    operand,
                    span,
                })
            }
            _ => self.parse_primary(core),
        }
    }
    
    /// Parse binary expression
    fn parse_binary(&mut self, core: &mut ParserCore, left: Expr) -> Result<Expr> {
        let op_token = core.advance().clone();
        let precedence = core.get_precedence(&op_token.kind);
        let right = self.parse_precedence(core, precedence.next())?;
        
        let op = core.token_to_binary_op(&op_token.kind)?;
        
        Ok(Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: op_token.span,
        })
    }
    
    /// Parse primary expression (literals, identifiers, parenthesized expressions)
    fn parse_primary(&mut self, core: &mut ParserCore) -> Result<Expr> {
        let token = core.advance();
        
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
                let expr = self.parse_expression(core)?;
                core.consume(&TokenKind::RightParen, "Expected ')'")?;
                Ok(expr)
            }
            _ => Err(Error::parser(
                format!("Unexpected token: {}", token.kind),
                token.span,
            )),
        }?;
        
        // Handle postfix expressions (calls, member access)
        expr = self.parse_postfix(core, expr)?;
        
        Ok(expr)
    }
    
    /// Parse postfix expressions (calls, member access)
    fn parse_postfix(&mut self, core: &mut ParserCore, mut expr: Expr) -> Result<Expr> {
        while !core.is_at_end() {
            match &core.peek().kind {
                TokenKind::LeftParen => {
                    // Function call: func(args)
                    let start_span = core.peek().span;
                    core.advance(); // consume '('
                    
                    let mut args = Vec::new();
                    while !core.check(&TokenKind::RightParen) && !core.is_at_end() {
                        args.push(self.parse_expression(core)?);
                        if !core.check(&TokenKind::RightParen) {
                            core.consume(&TokenKind::Comma, "Expected ',' between arguments")?;
                        }
                    }
                    
                    core.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
                    
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span: start_span,
                    };
                }
                TokenKind::Dot => {
                    // Member access: obj.prop
                    let start_span = core.peek().span;
                    core.advance(); // consume '.'
                    let property_name = core.consume_identifier("Expected property name after '.'")?;
                    
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
                    let start_span = core.peek().span;
                    core.advance(); // consume '['
                    let property = self.parse_expression(core)?;
                    core.consume(&TokenKind::RightBracket, "Expected ']'")?;
                    
                    expr = Expr::Member {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                        span: start_span,
                    };
                }
                TokenKind::PlusPlus => {
                    // Postfix increment: expr++
                    let span = core.peek().span;
                    core.advance(); // consume '++'
                    
                    expr = Expr::PostfixUnary {
                        op: PostfixUnaryOp::Increment,
                        operand: Box::new(expr),
                        span,
                    };
                }
                TokenKind::MinusMinus => {
                    // Postfix decrement: expr--
                    let span = core.peek().span;
                    core.advance(); // consume '--'
                    
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
}

impl ExpressionParserTrait for ExpressionParser {
    /// Parse expression using Pratt parsing
    fn parse_expression(&mut self, core: &mut ParserCore) -> Result<Expr> {
        self.parse_assignment(core)
    }
}
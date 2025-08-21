//! Statement parsing functionality
//! 
//! Handles parsing of all JavaScript statement types including declarations,
//! control flow statements, and block statements.

use super::ast::*;
use super::core::ParserCore;
use crate::error::{Error, Result};
use crate::lexer::{TokenKind, Keyword};

/// Trait for expression parsing capability
pub trait ExpressionParser {
    fn parse_expression(&mut self, core: &mut ParserCore) -> Result<Expr>;
}

/// Statement parser for handling all statement types
pub struct StatementParser;

impl StatementParser {
    /// Create a new statement parser
    pub fn new() -> Self {
        Self
    }
    
    /// Parse a statement
    pub fn parse_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        match &core.peek().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let_declaration(core, expr_parser),
            TokenKind::Keyword(Keyword::Var) => self.parse_var_declaration(core, expr_parser),
            TokenKind::Keyword(Keyword::Const) => self.parse_const_declaration(core, expr_parser),
            TokenKind::Keyword(Keyword::Function) => self.parse_function_declaration(core, expr_parser),
            TokenKind::Keyword(Keyword::If) => self.parse_if_statement(core, expr_parser),
            TokenKind::Keyword(Keyword::While) => self.parse_while_statement(core, expr_parser),
            TokenKind::Keyword(Keyword::For) => self.parse_for_statement(core, expr_parser),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_statement(core, expr_parser),
            TokenKind::LeftBrace => self.parse_block_statement(core, expr_parser),
            _ => self.parse_expression_statement(core, expr_parser),
        }
    }
    
    /// Parse let declaration: `let x = expr`
    fn parse_let_declaration<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'let'
        
        let name = core.consume_identifier("Expected variable name")?;
        
        let init = if core.match_token(&TokenKind::Equal) {
            Some(expr_parser.parse_expression(core)?)
        } else {
            None
        };
        
        core.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse var declaration: `var x = expr`
    fn parse_var_declaration<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'var'
        
        let name = core.consume_identifier("Expected variable name")?;
        
        let init = if core.match_token(&TokenKind::Equal) {
            Some(expr_parser.parse_expression(core)?)
        } else {
            None
        };
        
        core.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse const declaration: `const x = expr`
    fn parse_const_declaration<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'const'
        
        let name = core.consume_identifier("Expected variable name")?;
        
        if !core.match_token(&TokenKind::Equal) {
            return Err(Error::parser(
                "Missing initializer in const declaration".to_string(),
                core.peek().span,
            ));
        }
        
        let init = Some(expr_parser.parse_expression(core)?);
        
        core.consume_semicolon_or_newline();
        
        Ok(Stmt::VarDecl { 
            name, 
            init, 
            span: start_span 
        })
    }
    
    /// Parse function declaration: `function name(params) { body }`
    fn parse_function_declaration<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'function'
        
        let name = core.consume_identifier("Expected function name")?;
        
        core.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        
        while !core.check(&TokenKind::RightParen) && !core.is_at_end() {
            params.push(core.consume_identifier("Expected parameter name")?);
            if !core.check(&TokenKind::RightParen) {
                core.consume(&TokenKind::Comma, "Expected ',' between parameters")?;
            }
        }
        
        core.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;
        
        core.consume(&TokenKind::LeftBrace, "Expected '{' to start function body")?;
        let body = self.parse_block_statement_body(core, expr_parser)?;
        
        Ok(Stmt::FunctionDecl { name, params, body, span: start_span })
    }
    
    /// Parse if statement: `if (test) then_stmt else else_stmt`
    fn parse_if_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'if'
        
        core.consume(&TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let test = expr_parser.parse_expression(core)?;
        core.consume(&TokenKind::RightParen, "Expected ')' after if condition")?;
        
        let then_stmt = Box::new(self.parse_statement(core, expr_parser)?);
        
        let else_stmt = if core.match_token(&TokenKind::Keyword(Keyword::Else)) {
            Some(Box::new(self.parse_statement(core, expr_parser)?))
        } else {
            None
        };
        
        Ok(Stmt::If { test, then_stmt, else_stmt, span: start_span })
    }
    
    /// Parse while statement: `while (test) body`
    fn parse_while_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'while'
        
        core.consume(&TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let test = expr_parser.parse_expression(core)?;
        core.consume(&TokenKind::RightParen, "Expected ')' after while condition")?;
        
        let body = Box::new(self.parse_statement(core, expr_parser)?);
        
        Ok(Stmt::While { test, body, span: start_span })
    }
    
    /// Parse for statement: `for (init; test; update) body`
    fn parse_for_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'for'
        
        core.consume(&TokenKind::LeftParen, "Expected '(' after 'for'")?;
        
        // Parse init (can be a variable declaration or expression, or empty)
        let init = if core.match_token(&TokenKind::Semicolon) {
            None // Empty init
        } else if matches!(core.peek().kind, TokenKind::Keyword(Keyword::Var) | TokenKind::Keyword(Keyword::Let)) {
            // Variable declaration
            Some(Box::new(self.parse_statement(core, expr_parser)?))
        } else {
            // Expression statement
            let expr = expr_parser.parse_expression(core)?;
            core.consume(&TokenKind::Semicolon, "Expected ';' after for loop initializer")?;
            Some(Box::new(Stmt::Expression(expr)))
        };
        
        // Parse test condition (optional)
        let test = if core.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(expr_parser.parse_expression(core)?)
        };
        core.consume(&TokenKind::Semicolon, "Expected ';' after for loop condition")?;
        
        // Parse update expression (optional)
        let update = if core.check(&TokenKind::RightParen) {
            None
        } else {
            Some(expr_parser.parse_expression(core)?)
        };
        
        core.consume(&TokenKind::RightParen, "Expected ')' after for loop clauses")?;
        
        // Parse body
        let body = Box::new(self.parse_statement(core, expr_parser)?);
        
        Ok(Stmt::For { init, test, update, body, span: start_span })
    }
    
    /// Parse return statement: `return expr?;`
    fn parse_return_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.advance(); // consume 'return'
        
        let value = if matches!(core.peek().kind, TokenKind::Semicolon | TokenKind::Newline | TokenKind::Eof | TokenKind::RightBrace) {
            None
        } else {
            Some(expr_parser.parse_expression(core)?)
        };
        
        core.consume_semicolon_or_newline();
        
        Ok(Stmt::Return { value, span: start_span })
    }
    
    /// Parse block statement: `{ statements }`
    fn parse_block_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let start_span = core.peek().span;
        core.consume(&TokenKind::LeftBrace, "Expected '{'")?;
        
        let statements = self.parse_block_statement_body(core, expr_parser)?;
        
        Ok(Stmt::Block { statements, span: start_span })
    }
    
    /// Parse the body of a block (statements between braces)
    fn parse_block_statement_body<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !core.check(&TokenKind::RightBrace) && !core.is_at_end() {
            // Skip newlines and whitespace
            if matches!(core.peek().kind, TokenKind::Newline) {
                core.advance();
                continue;
            }
            
            // If we hit a closing brace, break out of the loop
            if core.check(&TokenKind::RightBrace) {
                break;
            }
            
            statements.push(self.parse_statement(core, expr_parser)?);
        }
        
        core.consume(&TokenKind::RightBrace, "Expected '}'")?;
        Ok(statements)
    }
    
    /// Parse expression statement: `expr;`
    fn parse_expression_statement<E: ExpressionParser>(
        &mut self, 
        core: &mut ParserCore, 
        expr_parser: &mut E
    ) -> Result<Stmt> {
        let expr = expr_parser.parse_expression(core)?;
        core.consume_semicolon_or_newline();
        Ok(Stmt::Expression(expr))
    }
}
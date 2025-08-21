//! JavaScript parser implementation
//! 
//! Orchestrates the three specialized parsing agents: ParserCore, StatementParser, and ExpressionParser.

use super::ast::*;
use super::core::ParserCore;
use super::statements::StatementParser;
use super::expressions::ExpressionParser;
use crate::error::Result;
use crate::lexer::{Token, TokenKind};

/// JavaScript parser - orchestrates the three specialized parsing agents
pub struct Parser {
    core: ParserCore,
    statement_parser: StatementParser,
    expression_parser: ExpressionParser,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            core: ParserCore::new(tokens),
            statement_parser: StatementParser::new(),
            expression_parser: ExpressionParser::new(),
        }
    }
    
    /// Parse the tokens into an AST
    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        
        while !self.core.is_at_end() {
            // Skip newlines
            if matches!(self.core.peek().kind, TokenKind::Newline) {
                self.core.advance();
                continue;
            }
            
            let stmt = self.statement_parser.parse_statement(
                &mut self.core, 
                &mut self.expression_parser
            )?;
            statements.push(stmt);
        }
        
        Ok(Program { statements })
    }
}
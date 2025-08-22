//! Implementation methods for AST nodes
//! 
//! This module provides method implementations for AST nodes, including
//! span access and Display formatting.

use std::fmt;
use crate::error::Span;
use super::nodes::{Program, Stmt, Expr, Property, PropertyKey};

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expression(expr) => expr.span(),
            Stmt::VarDecl { span, .. } => *span,
            Stmt::FunctionDecl { span, .. } => *span,
            Stmt::If { span, .. } => *span,
            Stmt::While { span, .. } => *span,
            Stmt::For { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Return { span, .. } => *span,
            Stmt::Break { span } => *span,
            Stmt::Continue { span } => *span,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_) => Span::new(0, 0, 1, 1), // TODO: Add span to literals
            Expr::Identifier { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::PostfixUnary { span, .. } => *span,
            Expr::Assignment { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Member { span, .. } => *span,
            Expr::Object { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::Function { span, .. } => *span,
            Expr::This { span } => *span,
        }
    }
}

// Display implementations for pretty printing

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Program {{")?;
        for stmt in &self.statements {
            writeln!(f, "  {}", stmt)?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "ExpressionStatement({})", expr),
            Stmt::VarDecl { name, init, .. } => {
                if let Some(init) = init {
                    write!(f, "VarDeclaration({} = {})", name, init)
                } else {
                    write!(f, "VarDeclaration({})", name)
                }
            }
            Stmt::FunctionDecl { name, params, body, .. } => {
                write!(f, "FunctionDeclaration({} ({}) {{ {} statements }})", 
                       name, params.join(", "), body.len())
            }
            Stmt::If { test, then_stmt, else_stmt, .. } => {
                if let Some(else_stmt) = else_stmt {
                    write!(f, "IfStatement({} then {} else {})", test, then_stmt, else_stmt)
                } else {
                    write!(f, "IfStatement({} then {})", test, then_stmt)
                }
            }
            Stmt::While { test, body, .. } => {
                write!(f, "WhileStatement({} do {})", test, body)
            }
            Stmt::For { init, test, update, body, .. } => {
                let init_str = if let Some(init) = init { format!("{}", init) } else { "".to_string() };
                let test_str = if let Some(test) = test { format!("{}", test) } else { "".to_string() };
                let update_str = if let Some(update) = update { format!("{}", update) } else { "".to_string() };
                write!(f, "ForStatement({};{};{} do {})", init_str, test_str, update_str, body)
            }
            Stmt::Block { statements, .. } => {
                write!(f, "BlockStatement({{ {} statements }})", statements.len())
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    write!(f, "ReturnStatement({})", value)
                } else {
                    write!(f, "ReturnStatement()")
                }
            }
            Stmt::Break { .. } => write!(f, "BreakStatement"),
            Stmt::Continue { .. } => write!(f, "ContinueStatement"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Identifier { name, .. } => write!(f, "{}", name),
            Expr::Binary { op, left, right, .. } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Expr::Unary { op, operand, .. } => {
                write!(f, "({}{})", op, operand)
            }
            Expr::PostfixUnary { op, operand, .. } => {
                write!(f, "({}{})", operand, op)
            }
            Expr::Assignment { left, right, .. } => {
                write!(f, "({} = {})", left, right)
            }
            Expr::Call { callee, args, .. } => {
                write!(f, "{}({})", callee, 
                       args.iter().map(|a| format!("{}", a)).collect::<Vec<_>>().join(", "))
            }
            Expr::Member { object, property, computed, .. } => {
                if *computed {
                    write!(f, "{}[{}]", object, property)
                } else {
                    write!(f, "{}.{}", object, property)
                }
            }
            Expr::Object { properties, .. } => {
                write!(f, "{{ {} }}", 
                       properties.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(", "))
            }
            Expr::Array { elements, .. } => {
                write!(f, "[{}]", 
                       elements.iter().map(|e| 
                           if let Some(e) = e { format!("{}", e) } else { "".to_string() }
                       ).collect::<Vec<_>>().join(", "))
            }
            Expr::Function { name, params, body, .. } => {
                let name_str = if let Some(name) = name { format!(" {}", name) } else { "".to_string() };
                write!(f, "function{}({}) {{ {} statements }}", 
                       name_str, params.join(", "), body.len())
            }
            Expr::This { .. } => write!(f, "this"),
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.value)
    }
}

impl fmt::Display for PropertyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyKey::Identifier(name) => write!(f, "{}", name),
            PropertyKey::String(s) => write!(f, "\"{}\"", s),
            PropertyKey::Number(n) => write!(f, "{}", n),
            PropertyKey::Computed(expr) => write!(f, "[{}]", expr),
        }
    }
}
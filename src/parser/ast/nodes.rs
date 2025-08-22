//! Core AST node definitions for JavaScript
//! 
//! This module defines the fundamental AST nodes used to represent parsed JavaScript code.
//! These are pure data structures without implementation logic.

use crate::error::Span;
use super::literals::Literal;
use super::operators::{BinaryOp, UnaryOp, PostfixUnaryOp};

/// Top-level program containing a list of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// JavaScript statement
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    VarDecl {
        name: String,
        init: Option<Expr>,
        span: Span,
    },
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    If {
        test: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
        span: Span,
    },
    While {
        test: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    For {
        init: Option<Box<Stmt>>,
        test: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
        span: Span,
    },
    Block {
        statements: Vec<Stmt>,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
}

/// JavaScript expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier {
        name: String,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    PostfixUnary {
        op: PostfixUnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Assignment {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Member {
        object: Box<Expr>,
        property: Box<Expr>,
        computed: bool, // true for obj[prop], false for obj.prop
        span: Span,
    },
    Object {
        properties: Vec<Property>,
        span: Span,
    },
    Array {
        elements: Vec<Option<Expr>>,
        span: Span,
    },
    Function {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    This {
        span: Span,
    },
}

/// Object property
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub key: PropertyKey,
    pub value: Expr,
    pub span: Span,
}

/// Object property key
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    Identifier(String),
    String(String),
    Number(f64),
    Computed(Expr),
}
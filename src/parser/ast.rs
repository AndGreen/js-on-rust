//! Abstract Syntax Tree definitions for JavaScript
//! 
//! This module defines the AST nodes used to represent parsed JavaScript code.

use crate::error::Span;
use std::fmt;

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

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Comparison
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Logical
    LogicalAnd,
    LogicalOr,
    
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    
    // Other
    InstanceOf,
    In,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
    TypeOf,
    Void,
    Delete,
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

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Null => write!(f, "null"),
            Literal::Undefined => write!(f, "undefined"),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Power => "**",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::StrictEqual => "===",
            BinaryOp::StrictNotEqual => "!==",
            BinaryOp::Less => "<",
            BinaryOp::Greater => ">",
            BinaryOp::LessEqual => "<=",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::LogicalAnd => "&&",
            BinaryOp::LogicalOr => "||",
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            BinaryOp::LeftShift => "<<",
            BinaryOp::RightShift => ">>",
            BinaryOp::UnsignedRightShift => ">>>",
            BinaryOp::InstanceOf => "instanceof",
            BinaryOp::In => "in",
        };
        write!(f, "{}", op_str)
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            UnaryOp::Plus => "+",
            UnaryOp::Minus => "-",
            UnaryOp::LogicalNot => "!",
            UnaryOp::BitwiseNot => "~",
            UnaryOp::TypeOf => "typeof ",
            UnaryOp::Void => "void ",
            UnaryOp::Delete => "delete ",
        };
        write!(f, "{}", op_str)
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
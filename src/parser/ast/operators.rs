//! JavaScript operator definitions for AST
//! 
//! This module defines all JavaScript operators and their string representations.

use std::fmt;

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

/// Postfix unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostfixUnaryOp {
    Increment, // ++
    Decrement, // --
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

impl fmt::Display for PostfixUnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            PostfixUnaryOp::Increment => "++",
            PostfixUnaryOp::Decrement => "--",
        };
        write!(f, "{}", op_str)
    }
}
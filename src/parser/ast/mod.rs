//! Abstract Syntax Tree definitions for JavaScript
//! 
//! This module provides a decomposed AST structure organized into logical components:
//! - `nodes`: Core AST node definitions (Program, Stmt, Expr, Property, PropertyKey)
//! - `literals`: Literal value types (Literal enum)
//! - `operators`: JavaScript operators (BinaryOp, UnaryOp, PostfixUnaryOp)
//! - `impls`: Method implementations (span access, Display formatting)
//! - `pretty`: Pretty printing functionality (PrettyPrint trait)

pub mod nodes;
pub mod literals;
pub mod operators;
pub mod impls;
pub mod pretty;

// Re-export all public types for backward compatibility
pub use nodes::{Program, Stmt, Expr, Property, PropertyKey};
pub use literals::Literal;
pub use operators::{BinaryOp, UnaryOp, PostfixUnaryOp};
pub use pretty::PrettyPrint;
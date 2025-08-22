//! Pretty printing functionality for AST nodes
//! 
//! This module provides detailed tree-like formatting for AST nodes,
//! useful for debugging and visualization.

use super::nodes::{Program, Stmt, Expr, Property, PropertyKey};
use super::literals::Literal;

// PrettyPrint trait for detailed tree-like AST representation
pub trait PrettyPrint {
    fn pretty_print(&self, indent: usize) -> String;
}

impl PrettyPrint for Program {
    fn pretty_print(&self, indent: usize) -> String {
        let mut result = format!("{}Program {{\n", "  ".repeat(indent));
        for stmt in &self.statements {
            result.push_str(&format!("{}{},\n", "  ".repeat(indent + 1), stmt.pretty_print(indent + 1)));
        }
        result.push_str(&format!("{}}}", "  ".repeat(indent)));
        result
    }
}

impl PrettyPrint for Stmt {
    fn pretty_print(&self, indent: usize) -> String {
        match self {
            Stmt::Expression(expr) => {
                format!("ExpressionStatement {{\n{}{}\n{}}}", 
                        "  ".repeat(indent + 1), 
                        expr.pretty_print(indent + 1),
                        "  ".repeat(indent))
            }
            Stmt::VarDecl { name, init, span } => {
                let mut result = format!("VarDeclaration {{\n{}name: \"{}\",\n", 
                                        "  ".repeat(indent + 1), name);
                if let Some(init) = init {
                    result.push_str(&format!("{}init: {},\n", 
                                           "  ".repeat(indent + 1), 
                                           init.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}init: None,\n", "  ".repeat(indent + 1)));
                }
                result.push_str(&format!("{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::FunctionDecl { name, params, body, span } => {
                let mut result = format!("FunctionDeclaration {{\n{}name: \"{}\",\n{}params: [{}],\n{}body: [\n", 
                                        "  ".repeat(indent + 1), name,
                                        "  ".repeat(indent + 1), params.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", "),
                                        "  ".repeat(indent + 1));
                for stmt in body {
                    result.push_str(&format!("{}{},\n", "  ".repeat(indent + 2), stmt.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::If { test, then_stmt, else_stmt, span } => {
                let mut result = format!("IfStatement {{\n{}test: {},\n{}then_stmt: {},\n", 
                                        "  ".repeat(indent + 1), test.pretty_print(indent + 1),
                                        "  ".repeat(indent + 1), then_stmt.pretty_print(indent + 1));
                if let Some(else_stmt) = else_stmt {
                    result.push_str(&format!("{}else_stmt: Some({}),\n", 
                                           "  ".repeat(indent + 1), else_stmt.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}else_stmt: None,\n", "  ".repeat(indent + 1)));
                }
                result.push_str(&format!("{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::While { test, body, span } => {
                format!("WhileStatement {{\n{}test: {},\n{}body: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), test.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), body.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Stmt::For { init, test, update, body, span } => {
                let mut result = format!("ForStatement {{\n", );
                if let Some(init) = init {
                    result.push_str(&format!("{}init: Some({}),\n", 
                                           "  ".repeat(indent + 1), init.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}init: None,\n", "  ".repeat(indent + 1)));
                }
                if let Some(test) = test {
                    result.push_str(&format!("{}test: Some({}),\n", 
                                           "  ".repeat(indent + 1), test.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}test: None,\n", "  ".repeat(indent + 1)));
                }
                if let Some(update) = update {
                    result.push_str(&format!("{}update: Some({}),\n", 
                                           "  ".repeat(indent + 1), update.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}update: None,\n", "  ".repeat(indent + 1)));
                }
                result.push_str(&format!("{}body: {},\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), body.pretty_print(indent + 1),
                               "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::Block { statements, span } => {
                let mut result = format!("BlockStatement {{\n{}statements: [\n", "  ".repeat(indent + 1));
                for stmt in statements {
                    result.push_str(&format!("{}{},\n", "  ".repeat(indent + 2), stmt.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::Return { value, span } => {
                let mut result = format!("ReturnStatement {{\n", );
                if let Some(value) = value {
                    result.push_str(&format!("{}value: Some({}),\n", 
                                           "  ".repeat(indent + 1), value.pretty_print(indent + 1)));
                } else {
                    result.push_str(&format!("{}value: None,\n", "  ".repeat(indent + 1)));
                }
                result.push_str(&format!("{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Stmt::Break { span } => {
                format!("BreakStatement {{\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Stmt::Continue { span } => {
                format!("ContinueStatement {{\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
        }
    }
}

impl PrettyPrint for Expr {
    fn pretty_print(&self, indent: usize) -> String {
        match self {
            Expr::Literal(lit) => {
                format!("Literal({})", lit.pretty_print(indent))
            }
            Expr::Identifier { name, span } => {
                format!("Identifier {{\n{}name: \"{}\",\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), name, "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::Binary { op, left, right, span } => {
                format!("BinaryExpression {{\n{}op: {:?},\n{}left: {},\n{}right: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), op,
                        "  ".repeat(indent + 1), left.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), right.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::Unary { op, operand, span } => {
                format!("UnaryExpression {{\n{}op: {:?},\n{}operand: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), op,
                        "  ".repeat(indent + 1), operand.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::PostfixUnary { op, operand, span } => {
                format!("PostfixUnaryExpression {{\n{}op: {:?},\n{}operand: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), op,
                        "  ".repeat(indent + 1), operand.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::Assignment { left, right, span } => {
                format!("AssignmentExpression {{\n{}left: {},\n{}right: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), left.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), right.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::Call { callee, args, span } => {
                let mut result = format!("CallExpression {{\n{}callee: {},\n{}args: [\n", 
                                        "  ".repeat(indent + 1), callee.pretty_print(indent + 1),
                                        "  ".repeat(indent + 1));
                for arg in args {
                    result.push_str(&format!("{}{},\n", "  ".repeat(indent + 2), arg.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Expr::Member { object, property, computed, span } => {
                format!("MemberExpression {{\n{}object: {},\n{}property: {},\n{}computed: {},\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), object.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), property.pretty_print(indent + 1),
                        "  ".repeat(indent + 1), computed,
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
            Expr::Object { properties, span } => {
                let mut result = format!("ObjectExpression {{\n{}properties: [\n", "  ".repeat(indent + 1));
                for prop in properties {
                    result.push_str(&format!("{}{},\n", "  ".repeat(indent + 2), prop.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Expr::Array { elements, span } => {
                let mut result = format!("ArrayExpression {{\n{}elements: [\n", "  ".repeat(indent + 1));
                for element in elements {
                    if let Some(element) = element {
                        result.push_str(&format!("{}Some({}),\n", "  ".repeat(indent + 2), element.pretty_print(indent + 2)));
                    } else {
                        result.push_str(&format!("{}None,\n", "  ".repeat(indent + 2)));
                    }
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Expr::Function { name, params, body, span } => {
                let mut result = format!("FunctionExpression {{\n", );
                if let Some(name) = name {
                    result.push_str(&format!("{}name: Some(\"{}\"),\n", "  ".repeat(indent + 1), name));
                } else {
                    result.push_str(&format!("{}name: None,\n", "  ".repeat(indent + 1)));
                }
                result.push_str(&format!("{}params: [{}],\n{}body: [\n", 
                                        "  ".repeat(indent + 1), params.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<_>>().join(", "),
                                        "  ".repeat(indent + 1)));
                for stmt in body {
                    result.push_str(&format!("{}{},\n", "  ".repeat(indent + 2), stmt.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}],\n{}span: {:?}\n{}}}", 
                               "  ".repeat(indent + 1), "  ".repeat(indent + 1), span, "  ".repeat(indent)));
                result
            }
            Expr::This { span } => {
                format!("ThisExpression {{\n{}span: {:?}\n{}}}", 
                        "  ".repeat(indent + 1), span, "  ".repeat(indent))
            }
        }
    }
}

impl PrettyPrint for Literal {
    fn pretty_print(&self, _indent: usize) -> String {
        match self {
            Literal::Number(n) => format!("Number({})", n),
            Literal::String(s) => format!("String(\"{}\")", s),
            Literal::Boolean(b) => format!("Boolean({})", b),
            Literal::Null => "Null".to_string(),
            Literal::Undefined => "Undefined".to_string(),
        }
    }
}

impl PrettyPrint for Property {
    fn pretty_print(&self, indent: usize) -> String {
        format!("Property {{\n{}key: {},\n{}value: {},\n{}span: {:?}\n{}}}", 
                "  ".repeat(indent + 1), self.key.pretty_print(indent + 1),
                "  ".repeat(indent + 1), self.value.pretty_print(indent + 1),
                "  ".repeat(indent + 1), self.span, "  ".repeat(indent))
    }
}

impl PrettyPrint for PropertyKey {
    fn pretty_print(&self, _indent: usize) -> String {
        match self {
            PropertyKey::Identifier(name) => format!("Identifier(\"{}\")", name),
            PropertyKey::String(s) => format!("String(\"{}\")", s),
            PropertyKey::Number(n) => format!("Number({})", n),
            PropertyKey::Computed(expr) => format!("Computed({})", expr.pretty_print(0)),
        }
    }
}
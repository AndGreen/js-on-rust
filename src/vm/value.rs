//! JavaScript Value Representation
//!
//! This module implements the dynamic value system for JavaScript,
//! supporting all primitive types and type coercion rules.

use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;

/// JavaScript value types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JavaScript number (always f64)
    Number(f64),
    /// JavaScript string
    String(Rc<String>),
    /// JavaScript boolean
    Boolean(bool),
    /// JavaScript null
    Null,
    /// JavaScript undefined
    Undefined,
    /// JavaScript object (placeholder for now)
    Object(Rc<ObjectData>),
    /// JavaScript function reference
    Function(FunctionRef),
}

/// Object data (simplified for now)
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectData {
    pub properties: HashMap<String, Value>,
}

/// Function reference
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionRef {
    /// Bytecode function index
    Bytecode(usize),
    /// Built-in function
    Native(NativeFunction),
}

/// Native function signature
pub type NativeFunction = fn(&[Value]) -> Value;

impl Value {
    /// Create a new number value
    pub fn number(n: f64) -> Self {
        Value::Number(n)
    }
    
    /// Create a new string value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(Rc::new(s.into()))
    }
    
    /// Create a new boolean value
    pub fn boolean(b: bool) -> Self {
        Value::Boolean(b)
    }
    
    /// Check if value is truthy (JavaScript truthiness rules)
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null | Value::Undefined => false,
            Value::Number(n) => !n.is_nan() && *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Object(_) | Value::Function(_) => true,
        }
    }
    
    /// Convert value to boolean (JavaScript ToBoolean)
    pub fn to_boolean(&self) -> bool {
        self.is_truthy()
    }
    
    /// Convert value to number (JavaScript ToNumber)
    pub fn to_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            Value::Boolean(true) => 1.0,
            Value::Boolean(false) => 0.0,
            Value::Null => 0.0,
            Value::Undefined => f64::NAN,
            Value::String(s) => {
                // Try to parse as number
                s.trim().parse::<f64>().unwrap_or(f64::NAN)
            }
            Value::Object(_) | Value::Function(_) => f64::NAN,
        }
    }
    
    /// Convert value to string (JavaScript ToString)
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.is_nan() {
                    "NaN".to_string()
                } else if n.is_infinite() {
                    if n.is_sign_negative() {
                        "-Infinity".to_string()
                    } else {
                        "Infinity".to_string()
                    }
                } else if n.fract() == 0.0 && n.abs() < 1e10 {
                    // Display integers without decimal point
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Undefined => "undefined".to_string(),
            Value::Object(_) => "[object Object]".to_string(),
            Value::Function(_) => "[object Function]".to_string(),
        }
    }
    
    /// Get the JavaScript typeof string
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "object", // typeof null is "object" in JavaScript
            Value::Undefined => "undefined",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
        }
    }
    
    /// Check for strict equality (===)
    pub fn strict_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                // NaN is not equal to itself
                if a.is_nan() || b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false, // Different types are never strictly equal
        }
    }
    
    /// Check for loose equality (==)
    pub fn loose_eq(&self, other: &Value) -> bool {
        // First check strict equality
        if self.strict_eq(other) {
            return true;
        }
        
        // Type coercion rules for ==
        match (self, other) {
            // null == undefined
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            
            // Number comparisons with coercion
            (Value::Number(n), other) | (other, Value::Number(n)) => {
                *n == other.to_number()
            }
            
            // Boolean to number coercion
            (Value::Boolean(b), other) | (other, Value::Boolean(b)) => {
                let n = if *b { 1.0 } else { 0.0 };
                Value::Number(n).loose_eq(other)
            }
            
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Undefined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_truthiness() {
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Undefined.is_truthy());
        assert!(Value::Number(1.0).is_truthy());
        assert!(!Value::Number(0.0).is_truthy());
        assert!(!Value::Number(f64::NAN).is_truthy());
        assert!(Value::string("hello").is_truthy());
        assert!(!Value::string("").is_truthy());
    }
    
    #[test]
    fn test_to_number() {
        assert_eq!(Value::Number(42.0).to_number(), 42.0);
        assert_eq!(Value::Boolean(true).to_number(), 1.0);
        assert_eq!(Value::Boolean(false).to_number(), 0.0);
        assert_eq!(Value::Null.to_number(), 0.0);
        assert!(Value::Undefined.to_number().is_nan());
        assert_eq!(Value::string("123").to_number(), 123.0);
        assert_eq!(Value::string("  456  ").to_number(), 456.0);
        assert!(Value::string("hello").to_number().is_nan());
    }
    
    #[test]
    fn test_to_string() {
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(3.14).to_string(), "3.14");
        assert_eq!(Value::Number(f64::NAN).to_string(), "NaN");
        assert_eq!(Value::Number(f64::INFINITY).to_string(), "Infinity");
        assert_eq!(Value::Number(f64::NEG_INFINITY).to_string(), "-Infinity");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Boolean(false).to_string(), "false");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Undefined.to_string(), "undefined");
        assert_eq!(Value::string("hello").to_string(), "hello");
    }
    
    #[test]
    fn test_type_of() {
        assert_eq!(Value::Number(42.0).type_of(), "number");
        assert_eq!(Value::string("hello").type_of(), "string");
        assert_eq!(Value::Boolean(true).type_of(), "boolean");
        assert_eq!(Value::Null.type_of(), "object"); // typeof null is "object"
        assert_eq!(Value::Undefined.type_of(), "undefined");
    }
    
    #[test]
    fn test_strict_equality() {
        assert!(Value::Number(42.0).strict_eq(&Value::Number(42.0)));
        assert!(!Value::Number(42.0).strict_eq(&Value::Number(43.0)));
        assert!(!Value::Number(f64::NAN).strict_eq(&Value::Number(f64::NAN))); // NaN !== NaN
        assert!(Value::string("hello").strict_eq(&Value::string("hello")));
        assert!(!Value::string("hello").strict_eq(&Value::string("world")));
        assert!(Value::Boolean(true).strict_eq(&Value::Boolean(true)));
        assert!(Value::Null.strict_eq(&Value::Null));
        assert!(Value::Undefined.strict_eq(&Value::Undefined));
        assert!(!Value::Number(0.0).strict_eq(&Value::Boolean(false))); // Different types
    }
    
    #[test]
    fn test_loose_equality() {
        assert!(Value::Number(42.0).loose_eq(&Value::Number(42.0)));
        assert!(Value::Number(0.0).loose_eq(&Value::Boolean(false)));
        assert!(Value::Number(1.0).loose_eq(&Value::Boolean(true)));
        assert!(Value::Null.loose_eq(&Value::Undefined));
        assert!(Value::Undefined.loose_eq(&Value::Null));
        assert!(Value::Number(123.0).loose_eq(&Value::string("123")));
        assert!(!Value::Number(123.0).loose_eq(&Value::string("456")));
    }
}
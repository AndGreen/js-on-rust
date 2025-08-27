//! Built-in JavaScript Functions
//!
//! This module implements native JavaScript functions like print, console.log,
//! and other essential runtime functions.

use std::collections::HashMap;
use super::value::{Value, NativeFunction};

/// Registry of built-in functions
pub struct Builtins {
    functions: HashMap<String, NativeFunction>,
}

impl Builtins {
    /// Create a new builtins registry with all standard functions
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // Register all built-in functions
        functions.insert("print".to_string(), print as NativeFunction);
        functions.insert("console.log".to_string(), console_log as NativeFunction);
        functions.insert("typeof".to_string(), typeof_fn as NativeFunction);
        functions.insert("isNaN".to_string(), is_nan as NativeFunction);
        functions.insert("isFinite".to_string(), is_finite as NativeFunction);
        functions.insert("parseInt".to_string(), parse_int as NativeFunction);
        functions.insert("parseFloat".to_string(), parse_float as NativeFunction);
        
        Self { functions }
    }
    
    /// Get a built-in function by name
    pub fn get(&self, name: &str) -> Option<NativeFunction> {
        self.functions.get(name).copied()
    }
    
    /// Check if a name is a built-in function
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Get all built-in function names
    pub fn names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}

// === Built-in Function Implementations ===

/// print(...args) - Print values to stdout
fn print(args: &[Value]) -> Value {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", output.join(" "));
    Value::Undefined
}

/// console.log(...args) - Print values to stdout (alias for print)
fn console_log(args: &[Value]) -> Value {
    print(args)
}

/// typeof(value) - Return the type of a value
fn typeof_fn(args: &[Value]) -> Value {
    match args.first() {
        Some(value) => Value::string(value.type_of()),
        None => Value::string("undefined"),
    }
}

/// isNaN(value) - Check if a value is NaN
fn is_nan(args: &[Value]) -> Value {
    match args.first() {
        Some(value) => {
            let num = value.to_number();
            Value::Boolean(num.is_nan())
        }
        None => Value::Boolean(true), // isNaN() with no args returns true
    }
}

/// isFinite(value) - Check if a value is finite
fn is_finite(args: &[Value]) -> Value {
    match args.first() {
        Some(value) => {
            let num = value.to_number();
            Value::Boolean(num.is_finite())
        }
        None => Value::Boolean(false), // isFinite() with no args returns false
    }
}

/// parseInt(string, radix) - Parse a string as an integer
fn parse_int(args: &[Value]) -> Value {
    let string = match args.first() {
        Some(v) => v.to_string(),
        None => return Value::Number(f64::NAN),
    };
    
    let radix = match args.get(1) {
        Some(v) => {
            let r = v.to_number();
            if r.is_nan() || r == 0.0 {
                10
            } else {
                r as i32
            }
        }
        None => 10,
    };
    
    // Validate radix
    if radix < 2 || radix > 36 {
        return Value::Number(f64::NAN);
    }
    
    // Trim whitespace and parse
    let trimmed = string.trim();
    if trimmed.is_empty() {
        return Value::Number(f64::NAN);
    }
    
    // Handle hex prefix for radix 16
    let (s, r) = if radix == 16 && (trimmed.starts_with("0x") || trimmed.starts_with("0X")) {
        (&trimmed[2..], 16)
    } else {
        (trimmed, radix)
    };
    
    // Parse the integer
    match i64::from_str_radix(s, r as u32) {
        Ok(n) => Value::Number(n as f64),
        Err(_) => {
            // Try parsing just the valid prefix
            let mut result = 0i64;
            let mut found_digit = false;
            
            for ch in s.chars() {
                if let Some(digit) = ch.to_digit(r as u32) {
                    result = result * (r as i64) + (digit as i64);
                    found_digit = true;
                } else {
                    break; // Stop at first invalid character
                }
            }
            
            if found_digit {
                Value::Number(result as f64)
            } else {
                Value::Number(f64::NAN)
            }
        }
    }
}

/// parseFloat(string) - Parse a string as a floating-point number
fn parse_float(args: &[Value]) -> Value {
    match args.first() {
        Some(v) => {
            let s = v.to_string();
            let trimmed = s.trim();
            
            if trimmed.is_empty() {
                return Value::Number(f64::NAN);
            }
            
            // JavaScript parseFloat is more lenient than Rust's parse
            // It parses as much as it can and ignores trailing garbage
            let mut end_idx = 0;
            let bytes = trimmed.as_bytes();
            let mut has_dot = false;
            let mut has_e = false;
            
            // Handle sign
            if bytes[0] == b'+' || bytes[0] == b'-' {
                end_idx = 1;
            }
            
            // Parse digits, decimal point, and exponent
            while end_idx < bytes.len() {
                let ch = bytes[end_idx];
                if ch.is_ascii_digit() {
                    end_idx += 1;
                } else if ch == b'.' && !has_dot && !has_e {
                    has_dot = true;
                    end_idx += 1;
                } else if (ch == b'e' || ch == b'E') && !has_e && end_idx > 0 {
                    has_e = true;
                    end_idx += 1;
                    // Handle exponent sign
                    if end_idx < bytes.len() && (bytes[end_idx] == b'+' || bytes[end_idx] == b'-') {
                        end_idx += 1;
                    }
                } else {
                    break;
                }
            }
            
            let valid_part = &trimmed[..end_idx];
            if valid_part.is_empty() || valid_part == "+" || valid_part == "-" {
                Value::Number(f64::NAN)
            } else {
                valid_part.parse::<f64>()
                    .map(Value::Number)
                    .unwrap_or(Value::Number(f64::NAN))
            }
        }
        None => Value::Number(f64::NAN),
    }
}

/// Console object implementation
pub struct Console;

impl Console {
    /// Create a console object value
    pub fn create_object() -> Value {
        use std::rc::Rc;
        use super::value::ObjectData;
        
        let mut properties = HashMap::new();
        
        // Add console methods as properties
        // For now, we'll just store them as undefined placeholders
        // In a full implementation, these would be function objects
        properties.insert("log".to_string(), Value::Undefined);
        properties.insert("error".to_string(), Value::Undefined);
        properties.insert("warn".to_string(), Value::Undefined);
        properties.insert("info".to_string(), Value::Undefined);
        properties.insert("debug".to_string(), Value::Undefined);
        
        Value::Object(Rc::new(ObjectData { properties }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_typeof() {
        assert_eq!(typeof_fn(&[Value::Number(42.0)]), Value::string("number"));
        assert_eq!(typeof_fn(&[Value::string("hello")]), Value::string("string"));
        assert_eq!(typeof_fn(&[Value::Boolean(true)]), Value::string("boolean"));
        assert_eq!(typeof_fn(&[Value::Null]), Value::string("object"));
        assert_eq!(typeof_fn(&[Value::Undefined]), Value::string("undefined"));
        assert_eq!(typeof_fn(&[]), Value::string("undefined"));
    }
    
    #[test]
    fn test_is_nan() {
        assert_eq!(is_nan(&[Value::Number(f64::NAN)]), Value::Boolean(true));
        assert_eq!(is_nan(&[Value::Number(42.0)]), Value::Boolean(false));
        assert_eq!(is_nan(&[Value::string("hello")]), Value::Boolean(true));
        assert_eq!(is_nan(&[Value::string("123")]), Value::Boolean(false));
        assert_eq!(is_nan(&[]), Value::Boolean(true));
    }
    
    #[test]
    fn test_is_finite() {
        assert_eq!(is_finite(&[Value::Number(42.0)]), Value::Boolean(true));
        assert_eq!(is_finite(&[Value::Number(f64::INFINITY)]), Value::Boolean(false));
        assert_eq!(is_finite(&[Value::Number(f64::NEG_INFINITY)]), Value::Boolean(false));
        assert_eq!(is_finite(&[Value::Number(f64::NAN)]), Value::Boolean(false));
        assert_eq!(is_finite(&[]), Value::Boolean(false));
    }
    
    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int(&[Value::string("123")]), Value::Number(123.0));
        assert_eq!(parse_int(&[Value::string("  456  ")]), Value::Number(456.0));
        assert_eq!(parse_int(&[Value::string("0xFF"), Value::Number(16.0)]), Value::Number(255.0));
        assert_eq!(parse_int(&[Value::string("1010"), Value::Number(2.0)]), Value::Number(10.0));
        assert_eq!(parse_int(&[Value::string("123abc")]), Value::Number(123.0));
        assert!(parse_int(&[Value::string("abc")]).to_number().is_nan());
    }
    
    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float(&[Value::string("3.14")]), Value::Number(3.14));
        assert_eq!(parse_float(&[Value::string("  -123.456  ")]), Value::Number(-123.456));
        assert_eq!(parse_float(&[Value::string("1.5e3")]), Value::Number(1500.0));
        assert_eq!(parse_float(&[Value::string("123abc")]), Value::Number(123.0));
        assert!(parse_float(&[Value::string("abc")]).to_number().is_nan());
    }
}
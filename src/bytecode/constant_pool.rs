//! Constant pool for bytecode functions
//!
//! This module manages constants referenced by bytecode instructions.
//! It provides deduplication to save memory and efficient access patterns.

use std::collections::HashMap;
use std::fmt;
use super::instruction::ConstIndex;

/// Wrapper for f64 that implements Hash and Eq for HashMap usage
#[derive(Debug, Clone, Copy)]
pub struct HashableF64(pub f64);

impl PartialEq for HashableF64 {
    fn eq(&self, other: &Self) -> bool {
        // Handle NaN case - in JS, NaN !== NaN, but for deduplication we treat them as equal
        if self.0.is_nan() && other.0.is_nan() {
            true
        } else {
            self.0 == other.0
        }
    }
}

impl Eq for HashableF64 {}

impl std::hash::Hash for HashableF64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.0.is_nan() {
            // All NaN values hash to the same value for deduplication
            std::f64::NAN.to_bits().hash(state);
        } else {
            self.0.to_bits().hash(state);
        }
    }
}

impl std::fmt::Display for HashableF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.0;
        if n.is_nan() {
            write!(f, "NaN")
        } else if n.is_infinite() {
            if n.is_sign_positive() {
                write!(f, "Infinity")
            } else {
                write!(f, "-Infinity")
            }
        } else if n.fract() == 0.0 && n as i64 as f64 == n {
            write!(f, "{}", n as i64)
        } else {
            write!(f, "{}", n)
        }
    }
}

/// Constant values that can be stored in the constant pool
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    /// JavaScript number (always f64 as per spec)
    Number(HashableF64),
    
    /// JavaScript string
    String(String),
    
    /// JavaScript boolean
    Boolean(bool),
    
    /// JavaScript null
    Null,
    
    /// JavaScript undefined
    Undefined,
    
    /// JavaScript regex pattern (stored as string for now)
    Regex(String),
    
    /// Property name for fast property access
    PropertyName(String),
}

impl fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstantValue::Number(n) => write!(f, "{}", n),
            ConstantValue::String(s) => write!(f, "\"{}\"", escape_string(s)),
            ConstantValue::Boolean(b) => write!(f, "{}", b),
            ConstantValue::Null => write!(f, "null"),
            ConstantValue::Undefined => write!(f, "undefined"),
            ConstantValue::Regex(pattern) => write!(f, "/{}/", pattern),
            ConstantValue::PropertyName(name) => write!(f, ".{}", name),
        }
    }
}

impl ConstantValue {
    /// Get the JavaScript type name for this constant
    pub fn type_name(&self) -> &'static str {
        match self {
            ConstantValue::Number(_) => "number",
            ConstantValue::String(_) | ConstantValue::PropertyName(_) => "string",
            ConstantValue::Boolean(_) => "boolean",
            ConstantValue::Null => "object", // typeof null === "object" in JS
            ConstantValue::Undefined => "undefined",
            ConstantValue::Regex(_) => "object",
        }
    }
    
    /// Check if this constant represents a truthy value in JavaScript
    pub fn is_truthy(&self) -> bool {
        match self {
            ConstantValue::Number(n) => !n.0.is_nan() && n.0 != 0.0,
            ConstantValue::String(s) | ConstantValue::PropertyName(s) => !s.is_empty(),
            ConstantValue::Boolean(b) => *b,
            ConstantValue::Null | ConstantValue::Undefined => false,
            ConstantValue::Regex(_) => true, // objects are always truthy
        }
    }
}

/// Pool of constants with deduplication and efficient lookup
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantPool {
    /// Vector of constant values (indexed by ConstIndex)
    values: Vec<ConstantValue>,
    
    /// Map for deduplication - maps constant to its index
    index_map: HashMap<ConstantValue, ConstIndex>,
    
    /// Cache for frequently accessed string constants
    string_cache: HashMap<String, ConstIndex>,
}

impl ConstantPool {
    /// Create a new empty constant pool
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            index_map: HashMap::new(),
            string_cache: HashMap::new(),
        }
    }
    
    /// Create a constant pool with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            index_map: HashMap::with_capacity(capacity),
            string_cache: HashMap::new(),
        }
    }
    
    /// Add a constant to the pool, returns its index
    /// If the constant already exists, returns the existing index (deduplication)
    pub fn add_constant(&mut self, value: ConstantValue) -> ConstIndex {
        // Check if we already have this constant
        if let Some(&index) = self.index_map.get(&value) {
            return index;
        }
        
        // Add new constant
        let index = self.values.len() as ConstIndex;
        
        // Handle potential overflow (should never happen in practice)
        if self.values.len() >= u16::MAX as usize {
            panic!("Constant pool overflow: too many constants (max {})", u16::MAX);
        }
        
        // Add to string cache if it's a string
        if let ConstantValue::String(ref s) = value {
            self.string_cache.insert(s.clone(), index);
        }
        
        self.values.push(value.clone());
        self.index_map.insert(value, index);
        
        index
    }
    
    /// Convenience method to add a number constant
    pub fn add_number(&mut self, n: f64) -> ConstIndex {
        self.add_constant(ConstantValue::Number(HashableF64(n)))
    }
    
    /// Convenience method to add a string constant
    pub fn add_string(&mut self, s: String) -> ConstIndex {
        // Check string cache first for better performance
        if let Some(&index) = self.string_cache.get(&s) {
            return index;
        }
        
        self.add_constant(ConstantValue::String(s))
    }
    
    /// Convenience method to add a boolean constant
    pub fn add_boolean(&mut self, b: bool) -> ConstIndex {
        self.add_constant(ConstantValue::Boolean(b))
    }
    
    /// Convenience method to add a property name
    pub fn add_property_name(&mut self, name: String) -> ConstIndex {
        self.add_constant(ConstantValue::PropertyName(name))
    }
    
    /// Add null constant
    pub fn add_null(&mut self) -> ConstIndex {
        self.add_constant(ConstantValue::Null)
    }
    
    /// Add undefined constant
    pub fn add_undefined(&mut self) -> ConstIndex {
        self.add_constant(ConstantValue::Undefined)
    }
    
    /// Get a constant by index
    pub fn get(&self, index: ConstIndex) -> Option<&ConstantValue> {
        self.values.get(index as usize)
    }
    
    /// Get a constant by index, panicking if not found
    pub fn get_unchecked(&self, index: ConstIndex) -> &ConstantValue {
        &self.values[index as usize]
    }
    
    /// Get the number of constants in the pool
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Check if the constant pool is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Get an iterator over all constants with their indices
    pub fn iter(&self) -> impl Iterator<Item = (ConstIndex, &ConstantValue)> {
        self.values
            .iter()
            .enumerate()
            .map(|(i, v)| (i as ConstIndex, v))
    }
    
    /// Find the index of a constant if it exists
    pub fn find_constant(&self, value: &ConstantValue) -> Option<ConstIndex> {
        self.index_map.get(value).copied()
    }
    
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> ConstantPoolStats {
        let values_size = std::mem::size_of_val(&self.values) +
            self.values.iter().map(|v| match v {
                ConstantValue::String(s) | ConstantValue::PropertyName(s) | ConstantValue::Regex(s) => s.capacity(),
                _ => 0,
            }).sum::<usize>();
        
        let index_map_size = self.index_map.capacity() * 
            (std::mem::size_of::<ConstantValue>() + std::mem::size_of::<ConstIndex>());
        
        let string_cache_size = self.string_cache.capacity() * 
            (std::mem::size_of::<String>() + std::mem::size_of::<ConstIndex>()) +
            self.string_cache.keys().map(|s| s.capacity()).sum::<usize>();
        
        ConstantPoolStats {
            constant_count: self.len(),
            values_size,
            index_map_size,
            string_cache_size,
            total_size: values_size + index_map_size + string_cache_size,
        }
    }
    
    /// Clear all constants (useful for testing)
    pub fn clear(&mut self) {
        self.values.clear();
        self.index_map.clear();
        self.string_cache.clear();
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about constant pool memory usage
#[derive(Debug, Clone)]
pub struct ConstantPoolStats {
    pub constant_count: usize,
    pub values_size: usize,
    pub index_map_size: usize,
    pub string_cache_size: usize,
    pub total_size: usize,
}

impl fmt::Display for ConstantPoolStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Constant Pool Stats:")?;
        writeln!(f, "  Constants: {}", self.constant_count)?;
        writeln!(f, "  Values size: {} bytes", self.values_size)?;
        writeln!(f, "  Index map size: {} bytes", self.index_map_size)?;
        writeln!(f, "  String cache size: {} bytes", self.string_cache_size)?;
        write!(f, "  Total size: {} bytes", self.total_size)
    }
}

impl fmt::Display for ConstantPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Constants [{}]:", self.len())?;
        for (i, value) in self.iter() {
            writeln!(f, "  #{}: {}", i, value)?;
        }
        Ok(())
    }
}

/// Helper function to escape string for display
fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            c if c.is_control() => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}
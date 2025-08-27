//! Call Stack Frame Management
//!
//! This module implements call frames for the VM's call stack,
//! managing local variables, return addresses, and function contexts.

use std::rc::Rc;
use super::value::Value;
use crate::bytecode::BytecodeFunction;

/// A call frame represents a function invocation on the call stack
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// The function being executed
    pub function: Rc<BytecodeFunction>,
    
    /// Instruction pointer (current bytecode offset)
    pub ip: usize,
    
    /// Base pointer for local variables on the stack
    pub stack_base: usize,
    
    /// Local variables storage
    pub locals: Vec<Value>,
    
    /// Return address (instruction pointer in calling function)
    pub return_address: Option<usize>,
    
    /// Calling frame's stack base (for restoring on return)
    pub caller_stack_base: Option<usize>,
}

impl CallFrame {
    /// Create a new call frame for the main program
    pub fn new_main(function: Rc<BytecodeFunction>) -> Self {
        let locals_count = function.locals_count as usize;
        Self {
            function,
            ip: 0,
            stack_base: 0,
            locals: vec![Value::Undefined; locals_count],
            return_address: None,
            caller_stack_base: None,
        }
    }
    
    /// Create a new call frame for a function call
    pub fn new_call(
        function: Rc<BytecodeFunction>,
        arguments: Vec<Value>,
        return_address: usize,
        caller_stack_base: usize,
        new_stack_base: usize,
    ) -> Self {
        let mut locals = vec![Value::Undefined; function.locals_count as usize];
        
        // Copy arguments to the first local slots
        let arity = function.arity as usize;
        let arg_count = arguments.len();
        
        for (i, arg) in arguments.into_iter().enumerate() {
            if i < arity {
                locals[i] = arg;
            } else {
                break; // Ignore extra arguments
            }
        }
        
        // Fill missing arguments with undefined
        for i in arg_count..arity {
            locals[i] = Value::Undefined;
        }
        
        Self {
            function,
            ip: 0,
            stack_base: new_stack_base,
            locals,
            return_address: Some(return_address),
            caller_stack_base: Some(caller_stack_base),
        }
    }
    
    /// Get a local variable by index
    pub fn get_local(&self, index: usize) -> Result<&Value, String> {
        self.locals
            .get(index)
            .ok_or_else(|| format!("Local variable index {} out of bounds", index))
    }
    
    /// Set a local variable by index
    pub fn set_local(&mut self, index: usize, value: Value) -> Result<(), String> {
        if index >= self.locals.len() {
            return Err(format!("Local variable index {} out of bounds", index));
        }
        self.locals[index] = value;
        Ok(())
    }
    
    /// Get the current instruction pointer
    pub fn ip(&self) -> usize {
        self.ip
    }
    
    /// Advance the instruction pointer by a given amount
    pub fn advance_ip(&mut self, amount: usize) {
        self.ip += amount;
    }
    
    /// Jump to a specific instruction
    pub fn jump_to(&mut self, target: usize) {
        self.ip = target;
    }
    
    /// Apply a relative jump offset
    pub fn jump_relative(&mut self, offset: i16) {
        if offset >= 0 {
            self.ip = self.ip.saturating_add(offset as usize);
        } else {
            self.ip = self.ip.saturating_sub((-offset) as usize);
        }
    }
    
    /// Check if we've reached the end of the bytecode
    pub fn is_at_end(&self) -> bool {
        self.ip >= self.function.bytecode.len()
    }
    
    /// Get the function name for debugging
    pub fn function_name(&self) -> &str {
        &self.function.name
    }
    
    /// Get debug information about the current position
    pub fn debug_info(&self) -> String {
        format!(
            "{}[{}] (locals: {}, stack_base: {})",
            self.function_name(),
            self.ip,
            self.locals.len(),
            self.stack_base
        )
    }
}

/// Call stack for managing nested function calls
#[derive(Debug)]
pub struct CallStack {
    /// Stack of call frames
    frames: Vec<CallFrame>,
    
    /// Maximum allowed call stack depth (to prevent stack overflow)
    max_depth: usize,
}

impl CallStack {
    /// Create a new call stack
    pub fn new(max_depth: usize) -> Self {
        Self {
            frames: Vec::new(),
            max_depth,
        }
    }
    
    /// Push a new frame onto the call stack
    pub fn push(&mut self, frame: CallFrame) -> Result<(), String> {
        if self.frames.len() >= self.max_depth {
            return Err("Maximum call stack depth exceeded".to_string());
        }
        self.frames.push(frame);
        Ok(())
    }
    
    /// Pop the top frame from the call stack
    pub fn pop(&mut self) -> Option<CallFrame> {
        self.frames.pop()
    }
    
    /// Get the current (top) frame
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.frames.last()
    }
    
    /// Get the current (top) frame mutably
    pub fn current_frame_mut(&mut self) -> Option<&mut CallFrame> {
        self.frames.last_mut()
    }
    
    /// Get the call stack depth
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
    
    /// Check if the call stack is empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    
    /// Generate a stack trace for debugging
    pub fn stack_trace(&self) -> Vec<String> {
        self.frames
            .iter()
            .rev()
            .map(|frame| {
                format!(
                    "  at {} (ip: {})",
                    frame.function_name(),
                    frame.ip()
                )
            })
            .collect()
    }
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new(1000) // Default maximum recursion depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_locals() {
        let func = Rc::new(BytecodeFunction::new("test".to_string(), 2, 5, 0));
        let args = vec![Value::Number(10.0), Value::Number(20.0)];
        let mut frame = CallFrame::new_call(func, args, 0, 0, 0);
        
        // Check arguments are properly set
        assert_eq!(frame.get_local(0).unwrap(), &Value::Number(10.0));
        assert_eq!(frame.get_local(1).unwrap(), &Value::Number(20.0));
        
        // Check other locals are undefined
        assert_eq!(frame.get_local(2).unwrap(), &Value::Undefined);
        
        // Test setting locals
        frame.set_local(2, Value::string("hello")).unwrap();
        assert_eq!(frame.get_local(2).unwrap(), &Value::string("hello"));
    }
    
    #[test]
    fn test_call_stack() {
        let mut stack = CallStack::new(10);
        assert!(stack.is_empty());
        
        let func = Rc::new(BytecodeFunction::new_main());
        let frame = CallFrame::new_main(func);
        stack.push(frame).unwrap();
        
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());
        
        let popped = stack.pop().unwrap();
        assert_eq!(popped.function_name(), "__main__");
        assert!(stack.is_empty());
    }
    
    #[test]
    fn test_max_call_depth() {
        let mut stack = CallStack::new(3);
        let func = Rc::new(BytecodeFunction::new_main());
        
        // Push 3 frames (should succeed)
        for _ in 0..3 {
            let frame = CallFrame::new_main(func.clone());
            stack.push(frame).unwrap();
        }
        
        // 4th frame should fail
        let frame = CallFrame::new_main(func);
        assert!(stack.push(frame).is_err());
    }
}
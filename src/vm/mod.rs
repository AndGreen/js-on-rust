//! Virtual Machine Module
//!
//! This module provides the JavaScript bytecode execution engine,
//! implementing a stack-based VM with accumulator register.

pub mod value;
pub mod frame;
pub mod builtins;
pub mod machine;

// Re-export main types
pub use value::{Value, FunctionRef, NativeFunction};
pub use frame::{CallFrame, CallStack};
pub use builtins::Builtins;
pub use machine::VM;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vm_creation() {
        let mut vm = VM::new();
        assert!(vm.execute(crate::bytecode::BytecodeFunction::new_main()).is_ok());
    }
}
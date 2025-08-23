//! Unit tests for bytecode system components
//!
//! These tests verify the core functionality of the bytecode system,
//! including instructions, constant pool, and function metadata.

use super::*;

#[cfg(test)]
mod instruction_tests {
    use super::*;
    
    #[test]
    fn test_instruction_display() {
        let instr = Bytecode::LdaConst(42);
        assert_eq!(format!("{}", instr), "LdaConst #42");
        
        let instr = Bytecode::Add;
        assert_eq!(format!("{}", instr), "Add");
        
        let instr = Bytecode::Jump(-5);
        assert_eq!(format!("{}", instr), "Jump -5");
    }
    
    #[test]
    fn test_instruction_analysis() {
        // Test control flow detection
        assert!(Bytecode::Jump(0).is_control_flow());
        assert!(Bytecode::JumpIfFalse(0).is_control_flow());
        assert!(Bytecode::Call(2).is_control_flow());
        assert!(Bytecode::Return.is_control_flow());
        assert!(!Bytecode::Add.is_control_flow());
        assert!(!Bytecode::LdaConst(0).is_control_flow());
        
        // Test accumulator modification
        assert!(Bytecode::LdaConst(0).modifies_accumulator());
        assert!(Bytecode::Add.modifies_accumulator());
        assert!(!Bytecode::StaLocal(0).modifies_accumulator());
        assert!(!Bytecode::Push.modifies_accumulator());
        
        // Test stack effects
        assert_eq!(Bytecode::Add.stack_pop_count(), 1);
        assert_eq!(Bytecode::Sub.stack_pop_count(), 1);
        assert_eq!(Bytecode::Call(3).stack_pop_count(), 4); // 3 args + function
        assert_eq!(Bytecode::Push.stack_push_count(), 1);
        assert_eq!(Bytecode::LdaConst(0).stack_pop_count(), 0);
    }
}

#[cfg(test)]
mod constant_pool_tests {
    use super::*;
    
    #[test]
    fn test_constant_pool_basic_operations() {
        let mut pool = ConstantPool::new();
        
        // Test adding different types
        let idx_num = pool.add_number(42.5);
        let idx_str = pool.add_string("hello".to_string());
        let idx_bool = pool.add_boolean(true);
        let idx_null = pool.add_null();
        
        // Check indices are sequential
        assert_eq!(idx_num, 0);
        assert_eq!(idx_str, 1);
        assert_eq!(idx_bool, 2);
        assert_eq!(idx_null, 3);
        
        // Check values can be retrieved
        assert!(matches!(pool.get(idx_num), Some(ConstantValue::Number(_))));
        assert!(matches!(pool.get(idx_str), Some(ConstantValue::String(_))));
        assert!(matches!(pool.get(idx_bool), Some(ConstantValue::Boolean(true))));
        assert!(matches!(pool.get(idx_null), Some(ConstantValue::Null)));
    }
    
    #[test]
    fn test_constant_deduplication() {
        let mut pool = ConstantPool::new();
        
        // Add same values multiple times
        let idx1 = pool.add_number(42.0);
        let idx2 = pool.add_number(42.0);
        let idx3 = pool.add_string("test".to_string());
        let idx4 = pool.add_string("test".to_string());
        
        // Should return same indices for same values
        assert_eq!(idx1, idx2);
        assert_eq!(idx3, idx4);
        
        // Pool should only contain unique values
        assert_eq!(pool.len(), 2);
    }
    
    #[test]
    fn test_constant_value_properties() {
        // Test truthy/falsy behavior
        assert!(ConstantValue::Number(HashableF64(42.0)).is_truthy());
        assert!(!ConstantValue::Number(HashableF64(0.0)).is_truthy());
        assert!(!ConstantValue::Number(HashableF64(f64::NAN)).is_truthy());
        
        assert!(ConstantValue::String("hello".to_string()).is_truthy());
        assert!(!ConstantValue::String("".to_string()).is_truthy());
        
        assert!(ConstantValue::Boolean(true).is_truthy());
        assert!(!ConstantValue::Boolean(false).is_truthy());
        
        assert!(!ConstantValue::Null.is_truthy());
        assert!(!ConstantValue::Undefined.is_truthy());
        
        // Test type names
        assert_eq!(ConstantValue::Number(HashableF64(42.0)).type_name(), "number");
        assert_eq!(ConstantValue::String("test".to_string()).type_name(), "string");
        assert_eq!(ConstantValue::Boolean(true).type_name(), "boolean");
        assert_eq!(ConstantValue::Null.type_name(), "object"); // JS quirk: typeof null === "object"
        assert_eq!(ConstantValue::Undefined.type_name(), "undefined");
    }
    
    #[test]
    fn test_hashable_f64() {
        let a = HashableF64(42.0);
        let b = HashableF64(42.0);
        let c = HashableF64(43.0);
        let nan1 = HashableF64(f64::NAN);
        let nan2 = HashableF64(f64::NAN);
        
        // Test equality
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(nan1, nan2); // NaN should equal NaN for deduplication
        
        // Test display formatting
        assert_eq!(format!("{}", HashableF64(42.0)), "42");
        assert_eq!(format!("{}", HashableF64(42.5)), "42.5");
        assert_eq!(format!("{}", HashableF64(f64::INFINITY)), "Infinity");
        assert_eq!(format!("{}", HashableF64(f64::NEG_INFINITY)), "-Infinity");
        assert_eq!(format!("{}", HashableF64(f64::NAN)), "NaN");
    }
}

#[cfg(test)]
mod bytecode_function_tests {
    use super::*;
    
    #[test]
    fn test_bytecode_function_creation() {
        let func = BytecodeFunction::new("test".to_string(), 2, 5, 10);
        
        assert_eq!(func.name, "test");
        assert_eq!(func.arity, 2);
        assert_eq!(func.locals_count, 5);
        assert_eq!(func.max_stack_size, 10);
        assert!(func.bytecode.is_empty());
        assert!(func.constants.is_empty());
    }
    
    #[test]
    fn test_main_function_creation() {
        let func = BytecodeFunction::new_main();
        
        assert_eq!(func.name, "<main>");
        assert_eq!(func.arity, 0);
    }
    
    #[test]
    fn test_instruction_addition() {
        let mut func = BytecodeFunction::new_main();
        
        func.add_instruction(Bytecode::LdaConst(0));
        func.add_instruction(Bytecode::Return);
        
        assert_eq!(func.bytecode.len(), 2);
        assert_eq!(func.current_offset(), 2);
        
        // Test getting instructions
        assert!(matches!(func.get_instruction(0), Some(Bytecode::LdaConst(0))));
        assert!(matches!(func.get_instruction(1), Some(Bytecode::Return)));
        assert!(func.get_instruction(2).is_none());
    }
    
    #[test]
    fn test_instruction_patching() {
        let mut func = BytecodeFunction::new_main();
        
        func.add_instruction(Bytecode::Jump(0)); // Placeholder
        func.add_instruction(Bytecode::Return);
        
        // Patch the jump instruction
        func.patch_instruction(0, Bytecode::Jump(5));
        
        assert!(matches!(func.get_instruction(0), Some(Bytecode::Jump(5))));
    }
    
    #[test]
    fn test_stack_size_calculation() {
        let mut func = BytecodeFunction::new_main();
        
        // Add instructions that affect stack
        func.add_instruction(Bytecode::Push); // +1 stack
        func.add_instruction(Bytecode::Push); // +1 stack (total: 2)
        func.add_instruction(Bytecode::Add);  // -1 stack (total: 1)
        func.add_instruction(Bytecode::Pop);  // -1 stack (total: 0)
        
        func.calculate_stack_size();
        
        assert_eq!(func.max_stack_size, 2); // Maximum reached was 2
    }
    
    #[test]
    fn test_function_signature() {
        let mut func = BytecodeFunction::new("test".to_string(), 2, 0, 0);
        
        assert_eq!(func.signature(), "function test(arg0, arg1)");
        
        func.set_flags(false, true, false);
        assert_eq!(func.signature(), "async function test(arg0, arg1)");
        
        func.set_flags(true, false, false);
        assert_eq!(func.signature(), "function* test(arg0, arg1)");
        
        func.set_flags(true, true, false);
        assert_eq!(func.signature(), "async function* test(arg0, arg1)");
    }
    
    #[test]
    fn test_function_stats() {
        let mut func = BytecodeFunction::new("test".to_string(), 1, 3, 0);
        
        func.add_instruction(Bytecode::LdaConst(0));
        func.add_instruction(Bytecode::JumpIfFalse(2));
        func.add_instruction(Bytecode::Return);
        
        let const_idx = func.constants.add_number(42.0);
        assert_eq!(const_idx, 0);
        
        let stats = func.stats();
        
        assert_eq!(stats.name, "test");
        assert_eq!(stats.instruction_count, 3);
        assert_eq!(stats.control_flow_count, 2); // JumpIfFalse and Return
        assert_eq!(stats.constants_count, 1);
        assert_eq!(stats.locals_count, 3);
        assert_eq!(stats.arity, 1);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_complete_function_workflow() {
        // Create a function representing: function add(a, b) { return a + b; }
        let mut func = BytecodeFunction::new("add".to_string(), 2, 2, 1);
        
        // Bytecode equivalent:
        // LdaLocal 0    // load parameter 'a'
        // LdaLocal 1    // load parameter 'b' 
        // Add           // a + b
        // Return        // return result
        
        func.add_instruction(Bytecode::LdaLocal(0));
        func.add_instruction(Bytecode::LdaLocal(1));
        func.add_instruction(Bytecode::Add);
        func.add_instruction(Bytecode::Return);
        
        func.calculate_stack_size();
        
        // Verify the function is constructed correctly
        assert_eq!(func.bytecode.len(), 4);
        assert_eq!(func.signature(), "function add(arg0, arg1)");
        assert_eq!(func.max_stack_size, 0); // No explicit stack usage
        
        // Test disassembly
        let disassembly = Disassembler::minimal_disassemble(&func);
        assert!(disassembly.contains("LdaLocal"));
        assert!(disassembly.contains("Add"));
        assert!(disassembly.contains("Return"));
    }
    
    #[test]
    fn test_function_with_constants() {
        let mut func = BytecodeFunction::new("greet".to_string(), 0, 1, 0);
        
        // Add constants
        let hello_idx = func.constants.add_string("Hello, ".to_string());
        let world_idx = func.constants.add_string("World!".to_string());
        
        // Generate bytecode: "Hello, " + "World!"
        func.add_instruction(Bytecode::LdaConst(hello_idx));
        func.add_instruction(Bytecode::LdaConst(world_idx));
        func.add_instruction(Bytecode::Add);
        func.add_instruction(Bytecode::Return);
        
        // Verify constants were deduplicated and stored
        assert_eq!(func.constants.len(), 2);
        
        // Verify disassembly includes constant values
        let disassembly = Disassembler::quick_disassemble(&func);
        assert!(disassembly.contains("Hello, "));
        assert!(disassembly.contains("World!"));
    }
}
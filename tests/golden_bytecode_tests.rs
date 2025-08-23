//! Golden tests for bytecode output
//!
//! These tests verify that the bytecode generation produces expected output
//! for known JavaScript inputs. This helps catch regressions in the compiler.

use std::fs;
use v8::{Engine, Result};

/// Test that simple arithmetic generates expected bytecode structure
#[test]
fn test_simple_arithmetic_bytecode() -> Result<()> {
    let source = fs::read_to_string("tests/golden/bytecode/simple_arithmetic.js")?;
    let mut engine = Engine::new_with_bytecode_debug();
    
    // Capture output (in real implementation, we'd capture the actual bytecode)
    // For now, just verify it compiles without error
    engine.execute(&source)?;
    
    // TODO: In Phase 2.2, we'll implement proper AST->Bytecode compilation
    // and can test against expected bytecode sequences
    
    Ok(())
}

/// Test that function declarations generate expected bytecode structure  
#[test]
fn test_function_call_bytecode() -> Result<()> {
    let source = fs::read_to_string("tests/golden/bytecode/function_call.js")?;
    let mut engine = Engine::new_with_bytecode_debug();
    
    // Verify compilation succeeds
    engine.execute(&source)?;
    
    // TODO: In Phase 2.2, verify actual function bytecode generation
    
    Ok(())
}

/// Test bytecode disassembly format consistency
#[test] 
fn test_bytecode_disassembly_format() {
    use v8::{BytecodeFunction, Bytecode, Disassembler};
    
    // Create a simple function manually
    let mut func = BytecodeFunction::new("test".to_string(), 2, 3, 1);
    
    // Add some constants and instructions
    let const_42 = func.constants.add_number(42.0);
    let const_hello = func.constants.add_string("Hello".to_string());
    
    func.add_instruction(Bytecode::LdaConst(const_42));
    func.add_instruction(Bytecode::StaLocal(0));
    func.add_instruction(Bytecode::LdaConst(const_hello));
    func.add_instruction(Bytecode::StaLocal(1));
    func.add_instruction(Bytecode::LdaLocal(0));
    func.add_instruction(Bytecode::LdaLocal(1));
    func.add_instruction(Bytecode::Add);
    func.add_instruction(Bytecode::Return);
    
    // Test disassembly format
    let disassembly = Disassembler::quick_disassemble(&func);
    
    // Verify expected format elements
    assert!(disassembly.contains("=== function test(arg0, arg1) ==="));
    assert!(disassembly.contains("Locals: 3, Max stack: 1"));
    assert!(disassembly.contains("Constants:"));
    assert!(disassembly.contains("#0: 42"));
    assert!(disassembly.contains("#1: \"Hello\""));
    assert!(disassembly.contains("Bytecode:"));
    assert!(disassembly.contains("LdaConst #0 (42)"));
    assert!(disassembly.contains("StaLocal 0"));
    assert!(disassembly.contains("Add"));
    assert!(disassembly.contains("Return"));
    
    // Verify instruction numbering
    assert!(disassembly.contains("0:"));
    assert!(disassembly.contains("7:"));
}

/// Test bytecode instruction correctness
#[test]
fn test_instruction_encoding_consistency() {
    use v8::Bytecode;
    
    // Test that instruction display is consistent
    let instructions = vec![
        Bytecode::LdaConst(42),
        Bytecode::LdaLocal(5),
        Bytecode::StaLocal(10),
        Bytecode::Add,
        Bytecode::Sub,
        Bytecode::Mul,
        Bytecode::Div,
        Bytecode::Eq,
        Bytecode::Lt,
        Bytecode::Jump(5),
        Bytecode::JumpIfFalse(-3),
        Bytecode::Call(2),
        Bytecode::Return,
        Bytecode::CreateObject,
    ];
    
    for (i, instr) in instructions.iter().enumerate() {
        let display = format!("{}", instr);
        
        // Verify no instruction displays as empty
        assert!(!display.is_empty(), "Instruction {} displays as empty", i);
        
        // Verify basic format expectations
        match instr {
            Bytecode::LdaConst(idx) => assert!(display.contains(&format!("#{}", idx))),
            Bytecode::LdaLocal(idx) | Bytecode::StaLocal(idx) => {
                assert!(display.contains(&idx.to_string()))
            }
            Bytecode::Jump(offset) | Bytecode::JumpIfFalse(offset) => {
                assert!(display.contains(&offset.to_string()))
            }
            Bytecode::Call(argc) => assert!(display.contains(&argc.to_string())),
            _ => {} // Other instructions just need non-empty display
        }
    }
}

/// Performance test: ensure bytecode structures are reasonably efficient
#[test]
fn test_bytecode_performance_characteristics() {
    use v8::BytecodeFunction;
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Create a moderately large function
    let mut func = BytecodeFunction::new("large_function".to_string(), 0, 100, 50);
    
    // Add many constants to test deduplication
    for i in 0..1000 {
        func.constants.add_number(i as f64);
        func.constants.add_string(format!("string_{}", i));
    }
    
    // Add duplicate constants to test deduplication efficiency
    for i in 0..100 {
        func.constants.add_number(i as f64); // Should be deduplicated
    }
    
    // Add many instructions
    for i in 0..1000 {
        func.add_instruction(v8::Bytecode::LdaConst(i % 100));
        func.add_instruction(v8::Bytecode::StaLocal((i % 100) as u16));
    }
    
    let elapsed = start.elapsed();
    
    // Verify deduplication worked
    assert_eq!(func.constants.len(), 2000); // 1000 numbers + 1000 strings, no duplicates
    
    // Verify reasonable performance (should complete in well under 100ms)
    assert!(elapsed.as_millis() < 100, "Bytecode creation took too long: {:?}", elapsed);
    
    // Test disassembly performance
    let start = Instant::now();
    let _disassembly = v8::Disassembler::quick_disassemble(&func);
    let elapsed = start.elapsed();
    
    assert!(elapsed.as_millis() < 50, "Disassembly took too long: {:?}", elapsed);
}
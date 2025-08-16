# Testing Strategy and Development Tools for V8-like JavaScript Engine

## Overview

This document outlines a comprehensive testing strategy and development tools suite for our V8-inspired JavaScript engine implemented in Rust. The testing approach covers all major components: parser, interpreter, JIT, GC, and inline caches, with emphasis on correctness validation and performance monitoring.

## 1. Component-Specific Testing Strategies

### 1.1 Frontend (Lexer/Parser) Testing

**Unit Tests:**
```rust
// tests/frontend/lexer_tests.rs
#[test]
fn test_tokenize_basic_expressions() {
    let input = "let x = 42 + y.prop";
    let tokens = tokenize(input);
    assert_eq!(tokens, vec![
        Token::Let, Token::Identifier("x"), Token::Assign,
        Token::Number(42.0), Token::Plus, Token::Identifier("y"),
        Token::Dot, Token::Identifier("prop")
    ]);
}

// tests/frontend/parser_tests.rs
#[test]
fn test_parse_function_declaration() {
    let source = "function add(a, b) { return a + b; }";
    let ast = parse(source).unwrap();
    match ast {
        Stmt::FunctionDecl { name, params, body } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
        }
        _ => panic!("Expected function declaration")
    }
}
```

**Property-Based Tests:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parse_roundtrip(source in any::<ValidJSSource>()) {
        let ast = parse(&source).unwrap();
        let regenerated = ast.to_string();
        let reparsed = parse(&regenerated).unwrap();
        assert_eq!(ast, reparsed);
    }
}
```

### 1.2 Bytecode Generation and Interpreter Testing

**Golden Tests:**
```rust
// tests/bytecode/golden_tests.rs
#[test]
fn test_bytecode_generation() {
    let test_cases = load_golden_test_cases("tests/golden/bytecode/");
    
    for case in test_cases {
        let ast = parse(&case.source).unwrap();
        let bytecode = compile_to_bytecode(ast);
        let actual = disassemble(&bytecode);
        
        assert_eq!(actual, case.expected_bytecode,
                  "Bytecode mismatch for: {}", case.name);
    }
}

// Golden test case format:
// tests/golden/bytecode/simple_assignment.js
let x = 42;

// tests/golden/bytecode/simple_assignment.expected
LdaConst 0    ; load 42
StaLocal 0    ; store to x
LdaUndefined  ; expression result
Return
```

**Interpreter Execution Tests:**
```rust
#[test]
fn test_interpreter_execution() {
    let vm = VM::new();
    let result = vm.execute("let x = 10; x * 2").unwrap();
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn test_control_flow() {
    let vm = VM::new();
    let code = r#"
        let result = 0;
        for (let i = 0; i < 5; i++) {
            result += i;
        }
        result;
    "#;
    assert_eq!(vm.execute(code).unwrap(), Value::Number(10.0));
}
```

### 1.3 Inline Caches Testing

**IC State Transition Tests:**
```rust
// tests/runtime/ic_tests.rs
#[test]
fn test_ic_state_transitions() {
    let vm = VM::new();
    
    // Setup objects with different shapes
    vm.execute(r#"
        function Point(x, y) { this.x = x; this.y = y; }
        function Circle(r) { this.radius = r; }
        
        let p1 = new Point(1, 2);
        let p2 = new Point(3, 4);
        let c1 = new Circle(5);
    "#);
    
    // Test monomorphic IC
    vm.execute("function getX(obj) { return obj.x; }");
    vm.execute("getX(p1); getX(p2);"); // Same shape - monomorphic
    
    let ic = vm.get_ic_state("getX", 0);
    assert_eq!(ic, IcState::Mono { shape: point_shape_id, slot: 0 });
    
    // Trigger polymorphic transition
    vm.execute("getX(c1);"); // Different shape
    let ic = vm.get_ic_state("getX", 0);
    assert!(matches!(ic, IcState::Poly(_)));
}

#[test]
fn test_ic_performance_characteristics() {
    let vm = VM::new();
    
    // Monomorphic case should be fastest
    let mono_time = benchmark_property_access(&vm, &[same_shape_objects]);
    
    // Polymorphic should be slower but reasonable
    let poly_time = benchmark_property_access(&vm, &[different_shape_objects]);
    
    // Megamorphic should be slowest
    let mega_time = benchmark_property_access(&vm, &[many_different_shapes]);
    
    assert!(mono_time < poly_time);
    assert!(poly_time < mega_time);
}
```

**IC Coverage Tests:**
```rust
#[derive(Debug)]
struct ICCoverageTracker {
    uninit_to_mono: usize,
    mono_to_poly: usize,
    poly_to_mega: usize,
    deoptimizations: usize,
}

#[test]
fn test_comprehensive_ic_coverage() {
    let mut tracker = ICCoverageTracker::default();
    let vm = VM::new_with_ic_tracker(&mut tracker);
    
    // Test all transition paths
    run_ic_transition_scenarios(&vm);
    
    assert!(tracker.uninit_to_mono > 0, "No uninit->mono transitions");
    assert!(tracker.mono_to_poly > 0, "No mono->poly transitions");
    assert!(tracker.poly_to_mega > 0, "No poly->mega transitions");
}
```

### 1.4 JIT Compiler Testing

**JIT Correctness Tests:**
```rust
#[test]
fn test_jit_compilation_correctness() {
    let vm = VM::new();
    
    // Hot function that should trigger JIT
    let code = r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        // Execute enough times to trigger JIT
        for (let i = 0; i < 1000; i++) {
            fibonacci(10);
        }
        
        fibonacci(15);
    "#;
    
    let interpreted_result = vm.execute_interpreted_only(code);
    let jit_result = vm.execute_with_jit(code);
    
    assert_eq!(interpreted_result, jit_result);
}

#[test]
fn test_deoptimization_correctness() {
    let vm = VM::new();
    
    vm.execute(r#"
        function polymorphic_add(a, b) {
            return a + b;  // JIT assumes number addition
        }
        
        // Train with numbers
        for (let i = 0; i < 1000; i++) {
            polymorphic_add(i, i + 1);
        }
        
        // Force deoptimization with string
        let result = polymorphic_add("hello", " world");
    "#);
    
    // Verify deoptimization happened and result is correct
    assert_eq!(vm.get_last_result(), Value::String("hello world".into()));
    assert!(vm.deoptimization_occurred());
}
```

**Deoptimization Tests:**
```rust
#[test]
fn test_deopt_stack_reconstruction() {
    let vm = VM::new();
    
    // Complex scenario with nested calls and deopt
    vm.execute(r#"
        function outer(x) {
            return inner(x) + 1;
        }
        
        function inner(x) {
            return x.value; // Type assumption here
        }
        
        // Train JIT
        for (let i = 0; i < 1000; i++) {
            outer({value: i});
        }
        
        // Force deopt in nested call
        try {
            outer(null); // Will deopt and throw
        } catch (e) {
            // Verify stack is correctly reconstructed
            let stack = e.stack;
            assert!(stack.includes("inner"));
            assert!(stack.includes("outer"));
        }
    "#);
}
```

### 1.5 Garbage Collector Testing

**Memory Safety Tests:**
```rust
#[test]
fn test_gc_basic_collection() {
    let mut vm = VM::new();
    let initial_heap_size = vm.heap_size();
    
    // Create many objects
    vm.execute(r#"
        let objects = [];
        for (let i = 0; i < 10000; i++) {
            objects.push({id: i, data: new Array(100)});
        }
    "#);
    
    let after_allocation = vm.heap_size();
    assert!(after_allocation > initial_heap_size);
    
    // Clear references
    vm.execute("objects = null;");
    
    // Force GC
    vm.force_gc();
    
    let after_gc = vm.heap_size();
    assert!(after_gc < after_allocation);
}

#[test]
fn test_generational_gc() {
    let mut vm = VM::new();
    
    // Test young generation collection
    for _ in 0..100 {
        vm.execute("let temp = {data: new Array(1000)};");
        // temp goes out of scope immediately
    }
    
    let young_collections = vm.young_gc_count();
    assert!(young_collections > 0);
    
    // Test promotion to old generation
    vm.execute(r#"
        let persistent = [];
        for (let i = 0; i < 1000; i++) {
            persistent.push({id: i});
        }
    "#);
    
    // Force several young collections
    for _ in 0..10 {
        vm.execute("let temp = new Array(1000);");
        vm.force_young_gc();
    }
    
    let old_objects = vm.old_generation_count();
    assert!(old_objects > 0);
}
```

**GC Stress Tests:**
```rust
#[test]
fn test_gc_stress_large_graphs() {
    let mut vm = VM::new();
    
    // Create complex object graph
    vm.execute(r#"
        function createGraph(depth, breadth) {
            if (depth <= 0) return {leaf: true};
            
            let node = {children: []};
            for (let i = 0; i < breadth; i++) {
                node.children.push(createGraph(depth - 1, breadth));
            }
            return node;
        }
        
        let graph = createGraph(10, 5);
    "#);
    
    // Stress test with many allocations during traversal
    vm.execute(r#"
        function traverse(node, depth = 0) {
            if (node.leaf) return depth;
            
            // Allocate during traversal to stress GC
            let temp = new Array(100);
            
            let maxDepth = depth;
            for (let child of node.children) {
                maxDepth = Math.max(maxDepth, traverse(child, depth + 1));
            }
            return maxDepth;
        }
        
        for (let i = 0; i < 100; i++) {
            traverse(graph);
        }
    "#);
    
    // Verify no crashes and reasonable memory usage
    assert!(vm.heap_size() < 100 * 1024 * 1024); // < 100MB
}

#[test]
fn test_gc_pause_times() {
    let mut vm = VM::new();
    let mut pause_times = Vec::new();
    
    vm.set_gc_pause_callback(|duration| {
        pause_times.push(duration);
    });
    
    // Create workload with mixed allocation patterns
    vm.execute(r#"
        for (let i = 0; i < 10000; i++) {
            // Short-lived objects
            let temp = {data: new Array(Math.random() * 1000)};
            
            // Some long-lived objects
            if (i % 100 === 0) {
                globalThis[`persistent_${i}`] = temp;
            }
        }
    "#);
    
    // Verify pause times are reasonable
    let max_pause = pause_times.iter().max().unwrap();
    let avg_pause = pause_times.iter().sum::<Duration>() / pause_times.len();
    
    assert!(max_pause < Duration::from_millis(100), "GC pause too long");
    assert!(avg_pause < Duration::from_millis(10), "Average pause too long");
}
```

## 2. Integration Testing

### 2.1 End-to-End Pipeline Tests

```rust
#[test]
fn test_complete_execution_pipeline() {
    let test_cases = [
        ("simple arithmetic", "2 + 3 * 4", Value::Number(14.0)),
        ("object property", "({x: 42}).x", Value::Number(42.0)),
        ("function call", "(function(x) { return x * 2; })(21)", Value::Number(42.0)),
        ("closure", r#"
            (function(x) {
                return function(y) { return x + y; };
            })(10)(32)
        "#, Value::Number(42.0)),
    ];
    
    for (name, code, expected) in test_cases {
        let vm = VM::new();
        let result = vm.execute(code).unwrap();
        assert_eq!(result, expected, "Test case '{}' failed", name);
    }
}
```

### 2.2 Cross-Component Integration

```rust
#[test]
fn test_jit_ic_gc_integration() {
    let mut vm = VM::new();
    
    // Test that JIT + IC + GC work together correctly
    vm.execute(r#"
        function processObjects(objects) {
            let total = 0;
            for (let obj of objects) {
                total += obj.value; // IC should optimize this
            }
            return total;
        }
        
        // Create many similar objects to train IC
        let objects = [];
        for (let i = 0; i < 10000; i++) {
            objects.push({value: i, metadata: new Array(10)});
        }
        
        // This should trigger JIT compilation
        for (let i = 0; i < 100; i++) {
            let result = processObjects(objects);
            
            // Create garbage to stress GC during JIT execution
            let garbage = new Array(1000);
        }
    "#);
    
    // Verify all systems worked correctly
    assert!(vm.jit_compilation_occurred());
    assert!(vm.ic_optimizations_applied());
    assert!(vm.gc_collections_performed() > 0);
}
```

## 3. Golden Tests Framework

### 3.1 Bytecode Golden Tests

```rust
// tests/golden/framework.rs
pub struct GoldenTestRunner {
    test_dir: PathBuf,
}

impl GoldenTestRunner {
    pub fn run_bytecode_tests(&self) -> Result<()> {
        let test_files = glob(&format!("{}/**/*.js", self.test_dir.display()))?;
        
        for test_file in test_files {
            let source = fs::read_to_string(&test_file)?;
            let expected_file = test_file.with_extension("expected");
            
            if expected_file.exists() {
                let expected = fs::read_to_string(expected_file)?;
                self.verify_bytecode_output(&source, &expected)?;
            } else {
                // Generate golden file
                let bytecode = self.compile_and_disassemble(&source)?;
                fs::write(expected_file, bytecode)?;
            }
        }
        Ok(())
    }
    
    fn verify_bytecode_output(&self, source: &str, expected: &str) -> Result<()> {
        let ast = parse(source)?;
        let bytecode = compile_to_bytecode(ast);
        let actual = disassemble(&bytecode);
        
        if actual.trim() != expected.trim() {
            return Err(format!(
                "Bytecode mismatch:\nExpected:\n{}\nActual:\n{}",
                expected, actual
            ).into());
        }
        Ok(())
    }
}
```

### 3.2 Output Golden Tests

```rust
// Test format:
// tests/golden/output/basic_operations.js
console.log(2 + 3);
console.log("hello" + " world");

// tests/golden/output/basic_operations.expected
5
hello world
```

## 4. Performance Testing and Benchmarking

### 4.1 Microbenchmarks

```rust
// benches/microbenchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_property_access(c: &mut Criterion) {
    let vm = VM::new();
    vm.execute(r#"
        let obj = {x: 1, y: 2, z: 3};
        function getX(o) { return o.x; }
    "#);
    
    c.bench_function("monomorphic property access", |b| {
        b.iter(|| vm.execute("getX(obj)"))
    });
}

fn benchmark_arithmetic(c: &mut Criterion) {
    let vm = VM::new();
    
    c.bench_function("integer addition", |b| {
        b.iter(|| vm.execute("1 + 2"))
    });
    
    c.bench_function("floating point operations", |b| {
        b.iter(|| vm.execute("3.14 * 2.718"))
    });
}

fn benchmark_function_calls(c: &mut Criterion) {
    let vm = VM::new();
    vm.execute("function identity(x) { return x; }");
    
    c.bench_function("function call overhead", |b| {
        b.iter(|| vm.execute("identity(42)"))
    });
}

criterion_group!(benches, 
    benchmark_property_access,
    benchmark_arithmetic, 
    benchmark_function_calls
);
criterion_main!(benches);
```

### 4.2 Macrobenchmarks

```rust
// benches/macrobenchmarks.rs
fn benchmark_fibonacci(c: &mut Criterion) {
    let vm = VM::new();
    vm.execute(r#"
        function fib(n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
    "#);
    
    c.bench_function("fibonacci(20)", |b| {
        b.iter(|| vm.execute("fib(20)"))
    });
}

fn benchmark_array_operations(c: &mut Criterion) {
    let vm = VM::new();
    
    c.bench_function("array creation and access", |b| {
        b.iter(|| vm.execute(r#"
            let arr = new Array(1000);
            for (let i = 0; i < 1000; i++) {
                arr[i] = i * 2;
            }
            let sum = 0;
            for (let i = 0; i < 1000; i++) {
                sum += arr[i];
            }
            sum;
        "#))
    });
}

fn benchmark_object_creation(c: &mut Criterion) {
    let vm = VM::new();
    
    c.bench_function("object creation and property access", |b| {
        b.iter(|| vm.execute(r#"
            let objects = [];
            for (let i = 0; i < 1000; i++) {
                objects.push({id: i, value: i * 2});
            }
            let total = 0;
            for (let obj of objects) {
                total += obj.value;
            }
            total;
        "#))
    });
}
```

## 5. Development Tools

### 5.1 Bytecode Disassembler

```rust
// tools/disassembler.rs
pub struct Disassembler {
    bytecode: Vec<u8>,
    constants: Vec<Value>,
}

impl Disassembler {
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        let mut pc = 0;
        
        while pc < self.bytecode.len() {
            let instruction = &self.bytecode[pc];
            let line = self.disassemble_instruction(instruction, &mut pc);
            output.push_str(&format!("{:04} {}\n", pc, line));
        }
        
        output
    }
    
    fn disassemble_instruction(&self, instruction: &u8, pc: &mut usize) -> String {
        match instruction {
            OP_LDA_CONST => {
                let idx = self.read_u16(pc);
                format!("LdaConst {} ; {}", idx, self.constants[idx as usize])
            }
            OP_STA_LOCAL => {
                let slot = self.read_u8(pc);
                format!("StaLocal {}", slot)
            }
            OP_LDA_LOCAL => {
                let slot = self.read_u8(pc);
                format!("LdaLocal {}", slot)
            }
            OP_ADD => "Add".to_string(),
            OP_CALL => {
                let argc = self.read_u8(pc);
                format!("Call {}", argc)
            }
            _ => format!("Unknown opcode: {}", instruction)
        }
    }
}

// CLI tool
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.js>", args[0]);
        return;
    }
    
    let source = fs::read_to_string(&args[1]).unwrap();
    let ast = parse(&source).unwrap();
    let (bytecode, constants) = compile_to_bytecode(ast);
    
    let disassembler = Disassembler::new(bytecode, constants);
    println!("{}", disassembler.disassemble());
}
```

### 5.2 Performance Profiler

```rust
// tools/profiler.rs
pub struct Profiler {
    function_stats: HashMap<String, FunctionStats>,
    ic_stats: HashMap<IcSite, IcStats>,
    gc_stats: GcStats,
}

#[derive(Debug)]
pub struct FunctionStats {
    call_count: u64,
    total_time: Duration,
    compiled_to_jit: bool,
    deoptimization_count: u64,
}

#[derive(Debug)]
pub struct IcStats {
    site_id: u32,
    property_name: String,
    transitions: Vec<IcTransition>,
    current_state: IcState,
}

impl Profiler {
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== FUNCTION PERFORMANCE ===\n");
        let mut functions: Vec<_> = self.function_stats.iter().collect();
        functions.sort_by_key(|(_, stats)| stats.total_time);
        functions.reverse();
        
        for (name, stats) in functions.iter().take(10) {
            report.push_str(&format!(
                "{:<30} calls: {:>8} time: {:>8}ms jit: {} deopt: {}\n",
                name, 
                stats.call_count,
                stats.total_time.as_millis(),
                if stats.compiled_to_jit { "yes" } else { "no" },
                stats.deoptimization_count
            ));
        }
        
        report.push_str("\n=== INLINE CACHE STATISTICS ===\n");
        let mono_count = self.ic_stats.values()
            .filter(|ic| matches!(ic.current_state, IcState::Mono { .. }))
            .count();
        let poly_count = self.ic_stats.values()
            .filter(|ic| matches!(ic.current_state, IcState::Poly(_)))
            .count();
        let mega_count = self.ic_stats.values()
            .filter(|ic| matches!(ic.current_state, IcState::Mega))
            .count();
            
        report.push_str(&format!(
            "Monomorphic: {:>6} ({:>5.1}%)\n",
            mono_count,
            100.0 * mono_count as f64 / self.ic_stats.len() as f64
        ));
        report.push_str(&format!(
            "Polymorphic: {:>6} ({:>5.1}%)\n",
            poly_count,
            100.0 * poly_count as f64 / self.ic_stats.len() as f64
        ));
        report.push_str(&format!(
            "Megamorphic: {:>6} ({:>5.1}%)\n",
            mega_count,
            100.0 * mega_count as f64 / self.ic_stats.len() as f64
        ));
        
        report.push_str("\n=== GARBAGE COLLECTION ===\n");
        report.push_str(&format!(
            "Young collections: {}\n", self.gc_stats.young_collections
        ));
        report.push_str(&format!(
            "Old collections: {}\n", self.gc_stats.old_collections
        ));
        report.push_str(&format!(
            "Average pause: {:.2}ms\n", 
            self.gc_stats.total_pause_time.as_millis() as f64 / 
            (self.gc_stats.young_collections + self.gc_stats.old_collections) as f64
        ));
        
        report
    }
}
```

### 5.3 Heap Dump Analyzer

```rust
// tools/heap_dump.rs
pub struct HeapDump {
    objects: Vec<HeapObject>,
    references: Vec<Reference>,
    roots: Vec<ObjectId>,
}

#[derive(Debug)]
pub struct HeapObject {
    id: ObjectId,
    size: usize,
    object_type: ObjectType,
    shape_id: Option<ShapeId>,
    properties: Vec<(String, Value)>,
}

impl HeapDump {
    pub fn analyze(&self) -> HeapAnalysis {
        let mut analysis = HeapAnalysis::default();
        
        // Analyze object distribution by type
        for obj in &self.objects {
            *analysis.type_distribution.entry(obj.object_type).or_insert(0) += 1;
            analysis.total_size += obj.size;
        }
        
        // Find largest objects
        analysis.largest_objects = self.objects.iter()
            .sorted_by_key(|obj| obj.size)
            .rev()
            .take(20)
            .cloned()
            .collect();
        
        // Detect potential memory leaks (unreachable cycles)
        analysis.potential_leaks = self.find_unreachable_cycles();
        
        // Shape analysis
        analysis.shape_stats = self.analyze_shapes();
        
        analysis
    }
    
    fn find_unreachable_cycles(&self) -> Vec<Vec<ObjectId>> {
        // Implement cycle detection algorithm
        // Mark all reachable objects from roots
        let mut reachable = HashSet::new();
        let mut stack = self.roots.clone();
        
        while let Some(obj_id) = stack.pop() {
            if reachable.insert(obj_id) {
                // Add referenced objects to stack
                for ref_edge in &self.references {
                    if ref_edge.from == obj_id {
                        stack.push(ref_edge.to);
                    }
                }
            }
        }
        
        // Find strongly connected components in unreachable subgraph
        // (implementation details omitted for brevity)
        vec![]
    }
}

// CLI tool
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <heap_dump.bin>", args[0]);
        return;
    }
    
    let dump = HeapDump::load(&args[1]).unwrap();
    let analysis = dump.analyze();
    
    println!("=== HEAP ANALYSIS ===");
    println!("Total objects: {}", dump.objects.len());
    println!("Total size: {} bytes", analysis.total_size);
    
    println!("\n=== TYPE DISTRIBUTION ===");
    for (obj_type, count) in &analysis.type_distribution {
        println!("{:?}: {}", obj_type, count);
    }
    
    if !analysis.potential_leaks.is_empty() {
        println!("\n=== POTENTIAL MEMORY LEAKS ===");
        for (i, cycle) in analysis.potential_leaks.iter().enumerate() {
            println!("Cycle {}: {} objects", i, cycle.len());
        }
    }
}
```

### 5.4 Interactive REPL

```rust
// tools/repl.rs
pub struct Repl {
    vm: VM,
    history: Vec<String>,
    completion_engine: CompletionEngine,
}

impl Repl {
    pub fn run(&mut self) {
        println!("JavaScript REPL v1.0");
        println!("Type .help for commands\n");
        
        let mut rl = Editor::<()>::new();
        
        loop {
            match rl.readline("js> ") {
                Ok(line) => {
                    let trimmed = line.trim();
                    
                    if trimmed.starts_with('.') {
                        self.handle_command(trimmed);
                    } else if !trimmed.is_empty() {
                        self.execute_line(trimmed);
                        rl.add_history_entry(&line);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Use .exit to quit");
                }
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }
    }
    
    fn handle_command(&mut self, command: &str) {
        match command {
            ".help" => self.show_help(),
            ".exit" => std::process::exit(0),
            ".clear" => self.vm = VM::new(),
            ".bytecode" => self.toggle_bytecode_display(),
            ".profile" => self.show_profile(),
            ".gc" => {
                self.vm.force_gc();
                println!("Garbage collection forced");
            }
            ".heap" => self.show_heap_stats(),
            ".ic" => self.show_ic_stats(),
            cmd if cmd.starts_with(".load ") => {
                let file = &cmd[6..];
                self.load_file(file);
            }
            _ => println!("Unknown command: {}", command),
        }
    }
    
    fn execute_line(&mut self, line: &str) {
        match self.vm.execute(line) {
            Ok(result) => {
                if !matches!(result, Value::Undefined) {
                    println!("{}", self.format_value(&result));
                }
            }
            Err(err) => println!("Error: {}", err),
        }
    }
    
    fn show_help(&self) {
        println!("Available commands:");
        println!("  .help      - Show this help");
        println!("  .exit      - Exit REPL");
        println!("  .clear     - Clear VM state");
        println!("  .bytecode  - Toggle bytecode display");
        println!("  .profile   - Show performance profile");
        println!("  .gc        - Force garbage collection");
        println!("  .heap      - Show heap statistics");
        println!("  .ic        - Show inline cache statistics");
        println!("  .load <file> - Load and execute file");
    }
}
```

### 5.5 JIT Visualization Tool

```rust
// tools/jit_visualizer.rs
pub struct JitVisualizer {
    compilation_events: Vec<CompilationEvent>,
    optimization_passes: Vec<OptimizationPass>,
}

#[derive(Debug)]
pub struct CompilationEvent {
    function_name: String,
    timestamp: Instant,
    tier: CompilationTier,
    input_bytecode: Vec<u8>,
    output_assembly: String,
    compilation_time: Duration,
}

impl JitVisualizer {
    pub fn generate_html_report(&self) -> String {
        let mut html = String::new();
        
        html.push_str(r#"
<!DOCTYPE html>
<html>
<head>
    <title>JIT Compilation Report</title>
    <style>
        .timeline { margin: 20px 0; }
        .function { margin: 10px 0; padding: 10px; border: 1px solid #ccc; }
        .baseline { background: #e6f3ff; }
        .optimized { background: #e6ffe6; }
        .deoptimized { background: #ffe6e6; }
        .assembly { font-family: monospace; white-space: pre; }
    </style>
</head>
<body>
    <h1>JIT Compilation Timeline</h1>
"#);
        
        for event in &self.compilation_events {
            let class = match event.tier {
                CompilationTier::Baseline => "baseline",
                CompilationTier::Optimized => "optimized",
                CompilationTier::Deoptimized => "deoptimized",
            };
            
            html.push_str(&format!(
                r#"<div class="function {}">
                    <h3>{} ({:?})</h3>
                    <p>Compiled in: {:?}</p>
                    <details>
                        <summary>Assembly Output</summary>
                        <div class="assembly">{}</div>
                    </details>
                </div>"#,
                class, event.function_name, event.tier,
                event.compilation_time, event.output_assembly
            ));
        }
        
        html.push_str("</body></html>");
        html
    }
}
```

## 6. Continuous Integration and Monitoring

### 6.1 CI Pipeline Configuration

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Run unit tests
      run: cargo test --lib
      
    - name: Run integration tests
      run: cargo test --test '*'
      
    - name: Run golden tests
      run: cargo test golden_tests
      
    - name: Check IC coverage
      run: cargo test ic_coverage_tests
      
    - name: Run benchmarks
      run: cargo bench
      
    - name: Generate test report
      run: |
        cargo test -- --format json > test_results.json
        python scripts/generate_test_report.py test_results.json
      
  performance:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    
    - name: Run performance regression tests
      run: |
        cargo bench -- --save-baseline main
        python scripts/check_performance_regression.py
```

### 6.2 Performance Monitoring

```rust
// tools/performance_monitor.rs
pub struct PerformanceMonitor {
    baseline_metrics: HashMap<String, f64>,
    current_metrics: HashMap<String, f64>,
    alerts: Vec<PerformanceAlert>,
}

#[derive(Debug)]
pub struct PerformanceAlert {
    metric_name: String,
    baseline_value: f64,
    current_value: f64,
    regression_percent: f64,
    severity: AlertSeverity,
}

impl PerformanceMonitor {
    pub fn check_regression(&mut self, threshold: f64) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        for (metric, &current) in &self.current_metrics {
            if let Some(&baseline) = self.baseline_metrics.get(metric) {
                let regression = (current - baseline) / baseline * 100.0;
                
                if regression > threshold {
                    alerts.push(PerformanceAlert {
                        metric_name: metric.clone(),
                        baseline_value: baseline,
                        current_value: current,
                        regression_percent: regression,
                        severity: if regression > threshold * 2.0 {
                            AlertSeverity::Critical
                        } else {
                            AlertSeverity::Warning
                        },
                    });
                }
            }
        }
        
        self.alerts.extend(alerts.clone());
        alerts
    }
}
```

## 7. Test Organization and Structure

```
tests/
├── unit/
│   ├── lexer/
│   ├── parser/
│   ├── bytecode/
│   ├── interpreter/
│   ├── jit/
│   ├── gc/
│   └── runtime/
├── integration/
│   ├── pipeline_tests.rs
│   ├── cross_component_tests.rs
│   └── regression_tests.rs
├── golden/
│   ├── bytecode/
│   │   ├── basic_arithmetic.js
│   │   ├── basic_arithmetic.expected
│   │   ├── control_flow.js
│   │   └── control_flow.expected
│   └── output/
│       ├── hello_world.js
│       └── hello_world.expected
├── performance/
│   ├── microbenchmarks/
│   ├── macrobenchmarks/
│   └── regression_tests/
└── tools/
    ├── test_runner.rs
    ├── golden_test_framework.rs
    └── performance_tracker.rs

benches/
├── microbenchmarks.rs
├── macrobenchmarks.rs
└── regression_benches.rs

tools/
├── disassembler.rs
├── profiler.rs
├── heap_dump.rs
├── repl.rs
├── jit_visualizer.rs
└── performance_monitor.rs
```

## 8. Key Metrics and KPIs

### 8.1 Correctness Metrics
- Test coverage: >95% line coverage, >90% branch coverage
- Golden test pass rate: 100%
- IC transition coverage: All state transitions tested
- Deoptimization correctness: 100% of deopt scenarios verified

### 8.2 Performance Metrics
- Microbenchmark performance: <5% regression tolerance
- Macrobenchmark performance: <10% regression tolerance
- JIT compilation speed: <100ms for typical functions
- GC pause times: <50ms for young generation, <200ms for old generation
- IC hit rates: >80% monomorphic, <10% megamorphic

### 8.3 Quality Metrics
- Memory leak detection: 0 leaks in stress tests
- Crash-free rate: >99.9% in extended testing
- Performance regression detection: Automated alerts
- Code review coverage: 100% of changes reviewed

This comprehensive testing strategy ensures the reliability, correctness, and performance of the V8-like JavaScript engine while providing developers with powerful tools for debugging, profiling, and optimization.
# Архитектурный план V8-подобного JavaScript движка на Rust

## Оглавление
1. [Общая архитектура](#общая-архитектура)
2. [Модульная структура](#модульная-структура)
3. [Компоненты системы](#компоненты-системы)
4. [Структуры данных](#структуры-данных)
5. [Интерфейсы между компонентами](#интерфейсы-между-компонентами)
6. [Потоки данных](#потоки-данных)
7. [JIT-компилятор и деоптимизация](#jit-компилятор-и-деоптимизация)
8. [Сборщик мусора](#сборщик-мусора)
9. [Этапы реализации](#этапы-реализации)

## Общая архитектура

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   JavaScript    │    │      Lexer       │    │     Parser      │
│     Source      │───▶│   (Tokenizer)    │───▶│   (AST Builder) │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
                                                         ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│    Garbage      │    │   Runtime (VM)   │    │   Bytecode      │
│   Collector     │◀──▶│  Interpreter     │◀───│   Generator     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         ▲                       │                       ▲
         │                       ▼                       │
         │              ┌─────────────────┐              │
         │              │   Profiler &    │              │
         │              │ Inline Caches   │              │
         │              └─────────────────┘              │
         │                       │                       │
         │                       ▼                       │
         │              ┌─────────────────┐              │
         │              │  Baseline JIT   │              │
         │              │   (Cranelift)   │              │
         │              └─────────────────┘              │
         │                       │                       │
         │                       ▼                       │
         │              ┌─────────────────┐              │
         │              │ Optimizing JIT  │              │
         └──────────────│   (TurboFan     │──────────────┘
                        │    style)       │
                        └─────────────────┘
                                 ▲ │
                                 │ ▼
                        ┌─────────────────┐
                        │ Deoptimization  │
                        │    System       │
                        └─────────────────┘
```

### Архитектурные принципы

1. **Tiered Compilation**: Интерпретатор → Baseline JIT → Оптимизирующий JIT
2. **Adaptive Optimization**: Профилирование + спекулятивные оптимизации
3. **Hidden Classes**: Быстрый доступ к свойствам объектов
4. **Inline Caches**: Кэширование результатов поиска свойств/методов
5. **Deoptimization**: Откат к безопасному коду при нарушении предположений
6. **Generational GC**: Эффективная сборка мусора для разных поколений объектов

## Модульная структура

### Крэйты и их зависимости

```
rustjs/
├── rustjs-lexer/           # Лексический анализатор
├── rustjs-parser/          # Синтаксический анализатор
├── rustjs-ir/              # Промежуточное представление и байткод
├── rustjs-runtime/         # Рантайм система (Value, Objects, Shapes)
├── rustjs-vm/              # Виртуальная машина и интерпретатор
├── rustjs-gc/              # Сборщик мусора
├── rustjs-jit/             # JIT компилятор
├── rustjs-std/             # Стандартная библиотека JS
├── rustjs-tools/           # Инструменты (REPL, профайлер)
└── rustjs/                 # Главный крэйт, связывающий все компоненты
```

#### Зависимости между крэйтами

```
rustjs-lexer
    ↓
rustjs-parser
    ↓
rustjs-ir ←─────────┐
    ↓               │
rustjs-runtime      │
    ↓               │
rustjs-vm ──────────┤
    ↓               │
rustjs-jit ─────────┘
    ↓
rustjs-gc
    ↓
rustjs-std
    ↓
rustjs-tools
    ↓
rustjs (main)
```

## Компоненты системы

### 1. Лексер (rustjs-lexer)

**Назначение**: Преобразование исходного кода JavaScript в поток токенов.

```rust
// Основные структуры
pub struct Lexer<'source> {
    source: &'source str,
    position: usize,
    line: u32,
    column: u32,
}

pub enum Token {
    // Литералы
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    
    // Идентификаторы и ключевые слова
    Identifier(String),
    Keyword(Keyword),
    
    // Операторы
    Plus, Minus, Star, Slash,
    Equal, EqualEqual, EqualEqualEqual,
    Bang, BangEqual, BangEqualEqual,
    Less, Greater, LessEqual, GreaterEqual,
    
    // Разделители
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Comma, Semicolon, Dot,
    
    // Специальные
    Eof,
    Error(String),
}

pub enum Keyword {
    Function, Return, If, Else, For, While,
    Const, Let, Var, True, False, Null, Undefined,
    Class, Extends, New, This, Super,
}
```

### 2. Парсер (rustjs-parser)

**Назначение**: Построение AST из потока токенов.

```rust
// AST узлы
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Member { object: Box<Expr>, property: Box<Expr>, computed: bool },
    Assignment { left: Box<Expr>, right: Box<Expr> },
    Object(Vec<Property>),
    Array(Vec<Expr>),
    Function(Function),
    This,
}

pub enum Stmt {
    Expression(Expr),
    VarDecl { name: String, init: Option<Expr> },
    FunctionDecl(Function),
    If { test: Expr, consequent: Box<Stmt>, alternate: Option<Box<Stmt>> },
    While { test: Expr, body: Box<Stmt> },
    For { init: Option<Box<Stmt>>, test: Option<Expr>, update: Option<Expr>, body: Box<Stmt> },
    Return(Option<Expr>),
    Block(Vec<Stmt>),
}

pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}
```

### 3. Байткод и IR (rustjs-ir)

**Назначение**: Генерация и представление байткода.

```rust
// Инструкции байткода (Ignition-style)
pub enum Instruction {
    // Загрузка/сохранение
    LdaConstant(u16),      // Load в аккумулятор
    LdaLocal(u16),         // Load локальную переменную
    StaLocal(u16),         // Store аккумулятор в локальную
    LdaGlobal(u16),        // Load глобальную
    StaGlobal(u16),        // Store в глобальную
    
    // Арифметика
    Add, Sub, Mul, Div, Mod,
    
    // Сравнения
    Equal, StrictEqual, Less, Greater, LessEqual, GreaterEqual,
    
    // Логические
    LogicalNot, LogicalAnd, LogicalOr,
    
    // Объекты и свойства
    LdaNamed(u16),         // obj.property
    StaNamed(u16),         // obj.property = value
    LdaKeyed,              // obj[key]
    StaKeyed,              // obj[key] = value
    
    // Вызовы
    Call(u8),              // Call с количеством аргументов
    CallMethod(u8, u16),   // obj.method()
    New(u8),               // new Constructor()
    
    // Управление потоком
    Jump(i16),             // Безусловный переход
    JumpIfTrue(i16),       // Переход если true
    JumpIfFalse(i16),      // Переход если false
    Return,                // Возврат из функции
    
    // Создание объектов
    CreateObject(u16),     // Создать объект
    CreateArray(u16),      // Создать массив
    CreateFunction(u16),   // Создать функцию
    
    // Отладка и профилирование
    DebugBreak,
    ProfileCall,
}

pub struct BytecodeFunction {
    pub name: String,
    pub param_count: u16,
    pub local_count: u16,
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub inline_caches: Vec<InlineCache>,
    pub debug_info: DebugInfo,
}
```

### 4. Рантайм система (rustjs-runtime)

**Назначение**: Представление значений, объектов и их метаданных.

```rust
// Основное представление значений
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(GcRef<JSString>),
    Object(GcRef<JSObject>),
    Function(GcRef<JSFunction>),
    Null,
    Undefined,
    Symbol(GcRef<JSSymbol>),
}

// JavaScript объект
pub struct JSObject {
    pub shape: ShapeId,
    pub elements: Elements,
    pub properties: Vec<Value>,
}

// Скрытые классы (Hidden Classes/Shapes)
pub struct Shape {
    pub id: ShapeId,
    pub transitions: HashMap<PropertyKey, ShapeId>,
    pub properties: Vec<PropertyDescriptor>,
    pub element_kind: ElementKind,
}

pub struct PropertyDescriptor {
    pub key: PropertyKey,
    pub offset: u16,
    pub attributes: PropertyAttributes,
}

// Элементы массивов
pub enum Elements {
    PackedSmi(Vec<i32>),           // Плотный массив малых целых
    PackedDouble(Vec<f64>),        // Плотный массив чисел
    PackedObject(Vec<Value>),      // Плотный массив объектов
    HoleyDouble(Vec<Option<f64>>), // Разреженный массив
    Dictionary(HashMap<u32, Value>), // Словарь для очень разреженных
}

// Inline Caches
pub enum ICState {
    Uninitialized,
    Monomorphic { shape: ShapeId, handler: ICHandler },
    Polymorphic(Vec<(ShapeId, ICHandler)>),
    Megamorphic,
}

pub enum ICHandler {
    LoadField(u16),        // Прямая загрузка поля
    StoreField(u16),       // Прямое сохранение поля
    CallBuiltin(BuiltinId), // Вызов встроенной функции
    CallOptimized(CodeRef), // Вызов оптимизированного кода
}
```

### 5. Виртуальная машина (rustjs-vm)

**Назначение**: Интерпретация байткода и управление выполнением.

```rust
pub struct VM {
    pub stack: Vec<Value>,
    pub call_stack: Vec<CallFrame>,
    pub globals: HashMap<String, Value>,
    pub shapes: ShapeTable,
    pub gc: GarbageCollector,
    pub profiler: Profiler,
}

pub struct CallFrame {
    pub function: GcRef<JSFunction>,
    pub instruction_pointer: usize,
    pub stack_base: usize,
    pub locals: Vec<Value>,
}

pub struct Interpreter {
    vm: VM,
}

impl Interpreter {
    pub fn execute(&mut self, function: &BytecodeFunction) -> Result<Value, RuntimeError> {
        let mut ip = 0;
        let mut accumulator = Value::Undefined;
        
        loop {
            match &function.instructions[ip] {
                Instruction::LdaConstant(idx) => {
                    accumulator = function.constants[*idx as usize].clone();
                }
                Instruction::Add => {
                    let rhs = self.vm.stack.pop().unwrap();
                    accumulator = self.add_values(accumulator, rhs)?;
                }
                // ... остальные инструкции
            }
            ip += 1;
        }
    }
}
```

### 6. JIT компилятор (rustjs-jit)

**Назначение**: Компиляция горячего кода в машинный код.

```rust
// Baseline JIT
pub struct BaselineCompiler {
    cranelift_context: Context,
    code_cache: CodeCache,
}

// Оптимизирующий JIT
pub struct OptimizingCompiler {
    sea_of_nodes: Graph,
    optimizations: Vec<OptimizationPass>,
}

// Узлы графа для оптимизирующего JIT
pub enum Node {
    Constant(Value),
    Parameter(u16),
    LoadField { object: NodeId, offset: u16 },
    StoreField { object: NodeId, value: NodeId, offset: u16 },
    Call { target: NodeId, args: Vec<NodeId> },
    Guard { condition: NodeId, deopt_info: DeoptInfo },
    Add(NodeId, NodeId),
    // ... другие операции
}

// Информация для деоптимизации
pub struct DeoptInfo {
    pub bytecode_offset: u32,
    pub local_values: Vec<ValueLocation>,
    pub stack_values: Vec<ValueLocation>,
}

pub enum ValueLocation {
    Register(RegId),
    Stack(i32),
    Constant(Value),
}
```

## Структуры данных

### Value Enum (оптимизированная версия)

```rust
// NaN-boxing для 64-битных систем
#[repr(transparent)]
pub struct Value(u64);

impl Value {
    const NAN_MASK: u64 = 0x7FF8000000000000;
    const TAG_MASK: u64 = 0x000F000000000000;
    
    // Теги для различных типов
    const TAG_BOOLEAN: u64 = 0x0001000000000000;
    const TAG_NULL: u64 = 0x0002000000000000;
    const TAG_UNDEFINED: u64 = 0x0003000000000000;
    const TAG_STRING: u64 = 0x0004000000000000;
    const TAG_OBJECT: u64 = 0x0005000000000000;
    const TAG_FUNCTION: u64 = 0x0006000000000000;
    
    pub fn new_number(n: f64) -> Self {
        Value(n.to_bits())
    }
    
    pub fn new_boolean(b: bool) -> Self {
        Value(Self::NAN_MASK | Self::TAG_BOOLEAN | if b { 1 } else { 0 })
    }
    
    pub fn is_number(&self) -> bool {
        (self.0 & Self::NAN_MASK) != Self::NAN_MASK
    }
    
    pub fn as_number(&self) -> Option<f64> {
        if self.is_number() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }
}
```

### Hidden Classes детально

```rust
pub struct ShapeTable {
    shapes: Vec<Shape>,
    root_shape: ShapeId,
    shape_cache: HashMap<ShapeTransition, ShapeId>,
}

pub struct ShapeTransition {
    from_shape: ShapeId,
    property: PropertyKey,
    attributes: PropertyAttributes,
}

impl ShapeTable {
    pub fn add_property(&mut self, shape_id: ShapeId, key: PropertyKey) -> ShapeId {
        let transition = ShapeTransition {
            from_shape: shape_id,
            property: key.clone(),
            attributes: PropertyAttributes::default(),
        };
        
        if let Some(&cached_shape) = self.shape_cache.get(&transition) {
            return cached_shape;
        }
        
        let current_shape = &self.shapes[shape_id.0];
        let mut new_properties = current_shape.properties.clone();
        new_properties.push(PropertyDescriptor {
            key,
            offset: new_properties.len() as u16,
            attributes: PropertyAttributes::default(),
        });
        
        let new_shape = Shape {
            id: ShapeId(self.shapes.len()),
            transitions: HashMap::new(),
            properties: new_properties,
            element_kind: current_shape.element_kind,
        };
        
        let new_shape_id = new_shape.id;
        self.shapes.push(new_shape);
        self.shape_cache.insert(transition, new_shape_id);
        
        new_shape_id
    }
}
```

### Inline Caches реализация

```rust
pub struct PropertyLoadIC {
    pub state: ICState,
    pub miss_count: u32,
}

impl PropertyLoadIC {
    pub fn load_property(&mut self, object: &JSObject, key: &PropertyKey) -> Option<Value> {
        match &self.state {
            ICState::Uninitialized => {
                // Первый вызов - инициализация IC
                if let Some(offset) = self.find_property_offset(object, key) {
                    self.state = ICState::Monomorphic {
                        shape: object.shape,
                        handler: ICHandler::LoadField(offset),
                    };
                    Some(object.properties[offset as usize].clone())
                } else {
                    None
                }
            }
            ICState::Monomorphic { shape, handler } => {
                if object.shape == *shape {
                    // Быстрый путь - shape совпал
                    match handler {
                        ICHandler::LoadField(offset) => {
                            Some(object.properties[*offset as usize].clone())
                        }
                        _ => None,
                    }
                } else {
                    // Shape не совпал - переход к полиморфному IC
                    self.transition_to_polymorphic(object, key)
                }
            }
            ICState::Polymorphic(handlers) => {
                // Поиск среди нескольких shape
                for (shape, handler) in handlers {
                    if object.shape == *shape {
                        match handler {
                            ICHandler::LoadField(offset) => {
                                return Some(object.properties[*offset as usize].clone())
                            }
                            _ => {}
                        }
                    }
                }
                // Не найден - добавляем новый handler или переходим к мегаморфному
                self.add_polymorphic_handler(object, key)
            }
            ICState::Megamorphic => {
                // Медленный путь - поиск в runtime
                self.slow_property_lookup(object, key)
            }
        }
    }
}
```

## Интерфейсы между компонентами

### 1. Lexer → Parser

```rust
pub trait TokenStream {
    fn next_token(&mut self) -> Result<Token, LexError>;
    fn peek_token(&self) -> Option<&Token>;
    fn position(&self) -> SourcePosition;
}
```

### 2. Parser → IR Generator

```rust
pub trait ASTVisitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> T;
    fn visit_function(&mut self, func: &Function) -> T;
}

pub struct BytecodeGenerator {
    instructions: Vec<Instruction>,
    constants: Vec<Value>,
    locals: HashMap<String, u16>,
}

impl ASTVisitor<()> for BytecodeGenerator {
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary { left, op, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
                self.emit_binary_op(*op);
            }
            // ... другие выражения
        }
    }
}
```

### 3. VM → JIT

```rust
pub trait CompilationTarget {
    fn should_compile(&self, function: &JSFunction, call_count: u32) -> bool;
    fn compile(&mut self, function: &JSFunction) -> Result<CompiledCode, JitError>;
}

pub struct HotspotDetector {
    call_counts: HashMap<FunctionId, u32>,
    compilation_threshold: u32,
}

impl HotspotDetector {
    pub fn record_call(&mut self, function_id: FunctionId) -> bool {
        let count = self.call_counts.entry(function_id).or_insert(0);
        *count += 1;
        *count >= self.compilation_threshold
    }
}
```

### 4. JIT → Deoptimization

```rust
pub trait DeoptimizationHandler {
    fn register_guard(&mut self, address: *const u8, deopt_info: DeoptInfo);
    fn deoptimize(&mut self, address: *const u8) -> Result<(), DeoptError>;
}

pub struct DeoptimizationManager {
    guard_map: HashMap<*const u8, DeoptInfo>,
    interpreter: *mut Interpreter,
}
```

## Потоки данных

### 1. Компиляция JavaScript в байткод

```
JavaScript Source
       ↓
   Lexer.tokenize()
       ↓
   Token Stream
       ↓
   Parser.parse()
       ↓
   AST (Expression/Statement tree)
       ↓
   IRGenerator.visit_ast()
       ↓
   Bytecode Instructions + Constants
       ↓
   BytecodeFunction
```

### 2. Выполнение байткода

```
BytecodeFunction
       ↓
   Interpreter.execute()
       ↓
   ┌─ Instruction Fetch
   │       ↓
   │   Decode & Execute
   │       ↓
   │   ┌─ Stack Operations
   │   ├─ Property Access (IC)
   │   ├─ Function Calls
   │   └─ Control Flow
   │       ↓
   └── Profiler.record()
       ↓
   Hot Code Detection
       ↓
   JIT Compilation Request
```

### 3. JIT-компиляция

```
Hot BytecodeFunction
       ↓
   BaselineCompiler.compile()
       ↓
   ┌─ CLIF IR Generation
   │       ↓
   │   Cranelift Optimization
   │       ↓
   │   Machine Code Generation
   │       ↓
   └── Install Guards & Deopt Info
       ↓
   Executable Native Code
       ↓
   CodeCache.store()
```

### 4. Управление объектами и GC

```
Object Creation
       ↓
   Shape Assignment
       ↓
   Heap Allocation (Young Space)
       ↓
   ┌─ Property Access (IC Update)
   │       ↓
   │   Shape Transition
   │       ↓
   └── Write Barrier (if needed)
       ↓
   GC Trigger Check
       ↓
   ┌─ Minor GC (Young → Old)
   │       ↓
   └── Major GC (Mark & Sweep)
       ↓
   Memory Compaction
```

## JIT-компилятор и деоптимизация

### Baseline JIT архитектура

```rust
pub struct BaselineCompiler {
    context: cranelift::Context,
    module: cranelift::Module<SimpleJitBackend>,
    ic_handlers: Vec<ICHandlerCode>,
}

impl BaselineCompiler {
    pub fn compile_function(&mut self, func: &BytecodeFunction) -> Result<CompiledFunction, JitError> {
        let mut builder = FunctionBuilder::new(&mut self.context.func, &mut self.context.build);
        
        // Создаем блоки для каждой инструкции
        let blocks = self.create_basic_blocks(&builder, func);
        
        // Генерируем код для каждой инструкции
        for (i, instruction) in func.instructions.iter().enumerate() {
            builder.switch_to_block(blocks[i]);
            
            match instruction {
                Instruction::LdaConstant(idx) => {
                    let value = &func.constants[*idx as usize];
                    let val = self.emit_load_constant(&mut builder, value);
                    self.store_accumulator(&mut builder, val);
                }
                Instruction::LdaNamed(idx) => {
                    // Генерируем код с IC
                    self.emit_property_load_with_ic(&mut builder, *idx, &func.inline_caches[*idx as usize]);
                }
                Instruction::Call(argc) => {
                    self.emit_call(&mut builder, *argc);
                }
                // ... другие инструкции
            }
        }
        
        // Финализируем функцию
        let compiled = self.module.define_function(func_id, &mut self.context)?;
        Ok(CompiledFunction {
            code_ptr: compiled.as_ptr(),
            deopt_info: self.generate_deopt_info(func),
        })
    }
    
    fn emit_property_load_with_ic(&mut self, builder: &mut FunctionBuilder, 
                                  property_idx: u16, ic: &InlineCache) -> Value {
        let object = self.load_accumulator(builder);
        
        // Генерируем проверку shape
        let shape_check = self.emit_shape_check(builder, object, ic.expected_shape);
        
        // Создаем два блока: fast_path и slow_path
        let fast_block = builder.create_block();
        let slow_block = builder.create_block();
        let merge_block = builder.create_block();
        
        builder.ins().brz(shape_check, slow_block, &[]);
        builder.ins().jump(fast_block, &[]);
        
        // Fast path - прямая загрузка
        builder.switch_to_block(fast_block);
        let fast_result = builder.ins().load(types::I64, MemFlags::new(), 
                                           object, ic.field_offset as i32);
        builder.ins().jump(merge_block, &[fast_result]);
        
        // Slow path - вызов runtime
        builder.switch_to_block(slow_block);
        let slow_result = self.emit_runtime_call(builder, "property_load", &[object]);
        builder.ins().jump(merge_block, &[slow_result]);
        
        // Merge
        builder.switch_to_block(merge_block);
        builder.append_block_param(merge_block, types::I64)
    }
}
```

### Оптимизирующий JIT (Sea of Nodes)

```rust
pub struct OptimizingCompiler {
    graph: Graph,
    nodes: Vec<Node>,
    value_numbering: HashMap<NodeSignature, NodeId>,
}

pub struct Graph {
    nodes: NodeArena,
    start_node: NodeId,
    end_node: NodeId,
    dominance_tree: DominanceTree,
}

impl OptimizingCompiler {
    pub fn optimize_function(&mut self, func: &BytecodeFunction) -> Result<OptimizedFunction, JitError> {
        // 1. Построение графа из байткода
        self.build_sea_of_nodes(func)?;
        
        // 2. Применение оптимизаций
        self.run_optimization_passes()?;
        
        // 3. Генерация машинного кода
        self.generate_optimized_code()
    }
    
    fn build_sea_of_nodes(&mut self, func: &BytecodeFunction) -> Result<(), JitError> {
        let mut builder = GraphBuilder::new(&mut self.graph);
        
        // Создаем узлы параметров
        for i in 0..func.param_count {
            builder.add_node(Node::Parameter(i));
        }
        
        // Проходим по инструкциям и строим граф
        for (ip, instruction) in func.instructions.iter().enumerate() {
            match instruction {
                Instruction::LdaConstant(idx) => {
                    let value = &func.constants[*idx as usize];
                    builder.add_node(Node::Constant(value.clone()));
                }
                Instruction::Add => {
                    let lhs = builder.current_accumulator();
                    let rhs = builder.pop_stack();
                    let add_node = builder.add_node(Node::Add(lhs, rhs));
                    builder.set_accumulator(add_node);
                }
                Instruction::LdaNamed(idx) => {
                    let object = builder.current_accumulator();
                    let load_node = builder.add_node(Node::LoadField {
                        object,
                        offset: self.resolve_property_offset(idx)?,
                    });
                    
                    // Добавляем guard для проверки shape
                    let guard_node = builder.add_node(Node::Guard {
                        condition: self.create_shape_check(object, idx),
                        deopt_info: DeoptInfo::new(ip, builder.current_state()),
                    });
                    
                    builder.add_dependency(load_node, guard_node);
                    builder.set_accumulator(load_node);
                }
                _ => {} // Другие инструкции
            }
        }
        
        Ok(())
    }
    
    fn run_optimization_passes(&mut self) -> Result<(), JitError> {
        // Global Value Numbering
        self.eliminate_common_subexpressions()?;
        
        // Constant Folding
        self.fold_constants()?;
        
        // Dead Code Elimination
        self.eliminate_dead_code()?;
        
        // Escape Analysis
        self.analyze_escapes()?;
        
        // Bounds Check Elimination
        self.eliminate_bounds_checks()?;
        
        Ok(())
    }
}
```

### Система деоптимизации

```rust
pub struct DeoptimizationInfo {
    pub bytecode_offset: u32,
    pub frame_state: FrameState,
    pub reason: DeoptReason,
}

pub struct FrameState {
    pub locals: Vec<ValueLocation>,
    pub stack: Vec<ValueLocation>,
    pub accumulator: ValueLocation,
}

pub enum DeoptReason {
    ShapeChanged,
    TypeMismatch,
    OverflowCheck,
    BoundsCheck,
    NullCheck,
}

pub struct DeoptimizationManager {
    deopt_points: HashMap<*const u8, DeoptimizationInfo>,
    interpreter: Box<dyn Interpreter>,
}

impl DeoptimizationManager {
    pub fn handle_deoptimization(&mut self, trap_address: *const u8) -> Result<Value, DeoptError> {
        let deopt_info = self.deopt_points.get(&trap_address)
            .ok_or(DeoptError::UnknownTrapSite)?;
        
        // Восстанавливаем состояние интерпретатора
        let frame_state = self.materialize_frame_state(&deopt_info.frame_state)?;
        
        // Возвращаемся к интерпретации с точки деоптимизации
        self.interpreter.resume_at_bytecode(
            deopt_info.bytecode_offset,
            frame_state
        )
    }
    
    fn materialize_frame_state(&self, frame_state: &FrameState) -> Result<InterpreterState, DeoptError> {
        let mut state = InterpreterState::new();
        
        // Восстанавливаем локальные переменные
        for (i, location) in frame_state.locals.iter().enumerate() {
            state.locals[i] = self.read_value_from_location(location)?;
        }
        
        // Восстанавливаем стек
        for location in &frame_state.stack {
            state.stack.push(self.read_value_from_location(location)?);
        }
        
        // Восстанавливаем аккумулятор
        state.accumulator = self.read_value_from_location(&frame_state.accumulator)?;
        
        Ok(state)
    }
    
    fn read_value_from_location(&self, location: &ValueLocation) -> Result<Value, DeoptError> {
        match location {
            ValueLocation::Register(reg) => {
                // Читаем из регистра (через контекст trap handler)
                self.read_register(*reg)
            }
            ValueLocation::Stack(offset) => {
                // Читаем из стека
                self.read_stack_slot(*offset)
            }
            ValueLocation::Constant(value) => {
                Ok(value.clone())
            }
        }
    }
}
```

## Сборщик мусора

### Генерационный GC архитектура

```rust
pub struct GarbageCollector {
    young_space: YoungGeneration,
    old_space: OldGeneration,
    write_barriers: WriteBarrierSet,
    allocation_rate: AllocationProfiler,
}

pub struct YoungGeneration {
    from_space: SemiSpace,
    to_space: SemiSpace,
    allocation_pointer: *mut u8,
    allocation_limit: *mut u8,
}

pub struct OldGeneration {
    pages: Vec<Page>,
    free_list: FreeList,
    mark_bits: BitVector,
    remembered_set: RememberedSet,
}

impl GarbageCollector {
    pub fn allocate_object(&mut self, size: usize, object_type: ObjectType) -> Result<*mut JSObject, AllocationError> {
        // Попытка аллокации в молодом поколении
        if let Ok(ptr) = self.young_space.try_allocate(size) {
            self.allocation_rate.record_allocation(size);
            return Ok(ptr);
        }
        
        // Молодое поколение заполнено - запускаем minor GC
        self.collect_young_generation()?;
        
        // Повторная попытка в молодом поколении
        if let Ok(ptr) = self.young_space.try_allocate(size) {
            return Ok(ptr);
        }
        
        // Если всё ещё не помещается - аллокация в старом поколении
        self.old_space.allocate(size)
    }
    
    pub fn collect_young_generation(&mut self) -> Result<(), GcError> {
        let mut evacuator = Evacuator::new(&mut self.young_space.to_space);
        
        // Сканируем корни
        self.scan_roots(&mut evacuator)?;
        
        // Сканируем remembered set (ссылки из старого поколения)
        self.scan_remembered_set(&mut evacuator)?;
        
        // Копируем живые объекты
        evacuator.process_evacuation_queue()?;
        
        // Обновляем указатели
        self.update_pointers_after_evacuation(&evacuator)?;
        
        // Меняем местами пространства
        std::mem::swap(&mut self.young_space.from_space, &mut self.young_space.to_space);
        
        // Сбрасываем to_space
        self.young_space.to_space.reset();
        
        Ok(())
    }
    
    pub fn collect_old_generation(&mut self) -> Result<(), GcError> {
        // Mark phase
        let mut marker = TriColorMarker::new(&mut self.old_space.mark_bits);
        self.mark_live_objects(&mut marker)?;
        
        // Sweep phase
        self.sweep_dead_objects()?;
        
        // Compact phase (если нужно)
        if self.should_compact() {
            self.compact_old_space()?;
        }
        
        Ok(())
    }
    
    fn mark_live_objects(&mut self, marker: &mut TriColorMarker) -> Result<(), GcError> {
        // Отмечаем корни как серые
        for root in self.scan_roots_for_marking() {
            marker.mark_grey(root);
        }
        
        // Обрабатываем серые объекты
        while let Some(object) = marker.pop_grey_object() {
            // Сканируем поля объекта
            for field in object.scan_fields() {
                if field.is_heap_pointer() {
                    marker.mark_grey(field.as_object_ptr());
                }
            }
            
            // Отмечаем объект как чёрный
            marker.mark_black(object);
        }
        
        Ok(())
    }
}

// Write Barriers для отслеживания межпоколенческих ссылок
pub struct WriteBarrierSet {
    remembered_set: CardTable,
}

impl WriteBarrierSet {
    pub fn record_write(&mut self, object: *mut JSObject, field_offset: usize, new_value: Value) {
        if self.is_cross_generational_write(object, &new_value) {
            self.remembered_set.mark_card(object);
        }
    }
    
    fn is_cross_generational_write(&self, object: *mut JSObject, value: &Value) -> bool {
        match value {
            Value::Object(obj_ref) => {
                self.is_old_generation(object) && self.is_young_generation(obj_ref.as_ptr())
            }
            _ => false
        }
    }
}
```

## Этапы реализации

### Фаза 1: Базовая инфраструктура (2-3 месяца)

1. **Лексер и парсер**
   - Поддержка базового подмножества JavaScript
   - Числа, строки, булевы значения, объекты, массивы
   - Базовые операторы и конструкции управления

2. **Байткод и интерпретатор**
   - Стековая виртуальная машина
   - Основные инструкции байткода
   - Простое представление Value (enum без оптимизаций)

3. **Базовые объекты**
   - Простые объекты без скрытых классов
   - Элементарный GC (mark & sweep без поколений)

### Фаза 2: Оптимизации доступа к данным (1-2 месяца)

1. **Hidden Classes (Shapes)**
   - Система переходов между shapes
   - Быстрый доступ к свойствам объектов

2. **Inline Caches**
   - Monomorphic и polymorphic IC
   - IC для property access и method calls

3. **Оптимизированные массивы**
   - Различные element kinds
   - Packed/holey массивы

### Фаза 3: JIT-компиляция (2-3 месяца)

1. **Baseline JIT**
   - Интеграция с Cranelift
   - Компиляция горячего байткода
   - Система профилирования

2. **Деоптимизация**
   - Guards в JIT-коде
   - Восстановление состояния интерпретатора
   - Обработка failed guards

### Фаза 4: Продвинутые оптимизации (3-4 месяца)

1. **Оптимизирующий JIT**
   - Sea of Nodes IR
   - Спекулятивные оптимизации
   - Inlining функций

2. **Продвинутый GC**
   - Генерационная сборка мусора
   - Write barriers и remembered sets
   - Инкрементальная маркировка

### Фаза 5: Расширение языка (2-3 месяца)

1. **Дополнительные языковые конструкции**
   - Классы и наследование
   - Замыкания и лексические области видимости
   - try/catch/finally

2. **Стандартная библиотека**
   - Встроенные объекты (Array, Object, String, Math)
   - Методы прототипов

Этот архитектурный план обеспечивает пошаговое построение современного JavaScript движка с использованием продвинутых техник оптимизации, принятых в V8, но адаптированных для экосистемы Rust.
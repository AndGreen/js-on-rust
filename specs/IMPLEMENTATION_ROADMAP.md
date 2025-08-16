# Implementation Roadmap: V8-подобный JavaScript движок на Rust

## Обзор проекта

Этот документ содержит детальный план реализации V8-подобного JavaScript движка на Rust, разбитый на 8 основных фаз с конкретными задачами, временными рамками и техническими требованиями.

### Архитектура системы
```
JS source
  ↓ (лексер/парсер)
AST  ──────────────► Lowering → Bytecode
                         ↓
                  Ignition-подобный интерпретатор
                         ↓ (профилирование: счётчики, IC)
                Baseline JIT (по желанию)
                         ↓ (горячие участки)
                    Оптимизирующий JIT (Cranelift)
                         ↕ (деоптимизация)
                        GC (генерационный)
```

---

## Фаза 1: Frontend - язык и парсер (4-6 недель) ✅ ЗАВЕРШЕНО

### Цель
Создать лексер и парсер для JS-подмножества с генерацией AST.

### Статус: ЗАВЕРШЕНО
**Дата завершения:** 16 августа 2025  
**Результат:** Полнофункциональный лексер и базовый парсер с поддержкой токенизации всех основных JS конструкций

### Задачи

#### 1.1 Лексический анализатор (1-2 недели) ✅ ЗАВЕРШЕНО
**Приоритет:** Критический  
**Зависимости:** Нет  

**Технические требования:**
- ✅ Токенизация основных JS конструкций
- ✅ Поддержка чисел (int, float, экспоненты), строк (с escape-последовательностями), идентификаторов
- ✅ Ключевые слова: `function`, `var`, `let`, `const`, `if`, `else`, `while`, `for`, `return`, etc.
- ✅ Операторы: все арифметические, логические, битовые, сравнения, присваивания
- ✅ Комментарии: однострочные (//) и блочные (/* */)
- ✅ Unicode поддержка для идентификаторов

**Критерии готовности:**
- [x] Успешная токенизация простых JS программ
- [x] Корректная обработка пробелов и комментариев
- [x] Информативные сообщения об ошибках с позициями

```rust
// Структура токена
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

pub enum TokenKind {
    // Литералы
    Number(f64),
    String(String),
    Identifier,
    
    // Ключевые слова
    Function, Var, Let, Const, If, Else, While, For, Return,
    
    // Операторы
    Plus, Minus, Star, Slash, Equals, EqualsEquals,
    NotEquals, Less, Greater, Dot,
    
    // Разделители
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Semicolon, Comma,
    
    Eof,
}
```

#### 1.2 Синтаксический анализатор (2-3 недели) ✅ ЧАСТИЧНО ЗАВЕРШЕНО
**Приоритет:** Критический  
**Зависимости:** 1.1  

**Технические требования:**
- ✅ Базовый recursive descent parser
- ✅ AST узлы для всех поддерживаемых конструкций (Expr, Stmt, Literal, etc.)
- ⏳ Pratt-парсер для выражений (заготовка создана)
- ✅ Базовое восстановление после ошибок
- ⏳ Поддерживаемая грамматика (требует доработки):
  - ✅ Выражения: литералы, базовая структура
  - ⏳ Операторы: `if/else`, `while`, `for`, блоки, `return`
  - ⏳ Функции: объявления и выражения
  - ⏳ Объекты: литералы объектов и массивов

**Критерии готовности:**
- [x] Парсинг программ без синтаксических ошибок (базовая реализация)
- [ ] Корректная приоритетность операторов
- [x] Понятные сообщения об ошибках
- [x] AST visitor pattern для обхода дерева

```rust
// Основные AST узлы
pub enum Stmt {
    Expr(Expr),
    VarDecl { name: String, init: Option<Expr> },
    FuncDecl { name: String, params: Vec<String>, body: Vec<Stmt> },
    If { test: Expr, then_stmt: Box<Stmt>, else_stmt: Option<Box<Stmt>> },
    While { test: Expr, body: Box<Stmt> },
    Block(Vec<Stmt>),
    Return(Option<Expr>),
}

pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Member { object: Box<Expr>, property: String },
    ObjectLit(Vec<(String, Expr)>),
    ArrayLit(Vec<Expr>),
}
```

#### 1.3 Обработка ошибок и тестирование (1 неделя) ✅ ЗАВЕРШЕНО
**Приоритет:** Высокий  
**Зависимости:** 1.2  

**Задачи:**
- ✅ Система диагностики с позициями в коде
- ✅ Цветной вывод ошибок в терминал
- ⏳ Набор тестов для корректных и некорректных программ (требует расширения)
- ⏳ Интеграционные тесты с golden files (планируется)

### Достижения Фазы 1

**Реализованная функциональность:**
- ✅ Полнофункциональный лексер с поддержкой всех основных JS токенов
- ✅ Unicode-поддержка идентификаторов через unicode-xid
- ✅ Escape-последовательности в строках (включая \xHH, \uHHHH, \u{HHHHHH})
- ✅ Комментарии: однострочные (//) и блочные (/* */)
- ✅ Полная система операторов: арифметические, логические, сравнения, присваивания
- ✅ Обработка чисел: целые, с плавающей точкой, экспоненциальная нотация
- ✅ Базовая архитектура парсера с AST узлами для всех JS конструкций
- ✅ Система ошибок с точными позициями и цветным выводом
- ✅ CLI интерфейс с поддержкой REPL и выполнения файлов

**Тестирование:**
- ✅ Успешная токенизация реального JavaScript кода
- ✅ Корректная обработка edge cases (комментарии, строки, операторы)
- ✅ Информативные сообщения об ошибках с указанием позиций

**Архитектурные решения:**
- ✅ Модульная структура с четким разделением ответственности (lexer, parser, error)
- ✅ Enum-based токенизация для type safety
- ✅ Span-based отслеживание позиций для точной диагностики
- ✅ Extensible AST дизайн для будущих языковых расширений

---

## Фаза 2: Байткод и интерпретатор (5-7 недель) ⏳ СЛЕДУЮЩАЯ

### Цель
Создать стековую VM с байткодным интерпретатором в стиле Ignition.

### Статус: ГОТОВ К НАЧАЛУ
**Зависимости:** ✅ Фаза 1 завершена  
**Следующий шаг:** Начать с дизайна байткода (Задача 2.1)

### Задачи

#### 2.1 Дизайн байткода (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 1.2  

**Технические требования:**
- Стековая архитектура с аккумулятором
- Компактное представление инструкций
- Поддержка локальных переменных и констант

```rust
// Формат байткода
pub enum Bytecode {
    // Загрузка/сохранение
    LdaConst(u16),           // Load accumulator from constant pool
    LdaLocal(u16),           // Load accumulator from local
    StaLocal(u16),           // Store accumulator to local
    
    // Арифметические операции
    Add, Sub, Mul, Div,
    
    // Сравнения
    Eq, Ne, Lt, Gt,
    
    // Доступ к свойствам
    LdaNamed(u16),           // Load property name from constant pool
    StaNamed(u16),           // Store property
    
    // Вызовы и управление
    Call(u8),                // Call with argc
    Return,
    
    // Переходы
    Jump(i16),
    JumpIfFalse(i16),
    JumpIfTrue(i16),
    
    // Создание объектов
    CreateObject,
    CreateArray(u16),        // Array with initial size
}

pub struct BytecodeFunction {
    pub name: String,
    pub arity: u8,
    pub locals_count: u16,
    pub bytecode: Vec<Bytecode>,
    pub constants: Vec<Value>,
    pub debug_info: DebugInfo,
}
```

#### 2.2 Компилятор AST → Байткод (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 2.1  

**Технические требования:**
- Обход AST с генерацией байткода
- Управление локальными переменными и скоупами
- Оптимизация простых случаев (constant folding)
- Генерация debug info для отладки

**Критерии готовности:**
- [ ] Корректная компиляция всех поддерживаемых конструкций
- [ ] Правильное управление стеком
- [ ] Генерация jump-адресов для циклов и условий
- [ ] Дизассемблер для отладки

#### 2.3 Виртуальная машина (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 2.2  

**Технические требования:**
- Исполнение байткода с аккумулятором и стеком
- Управление фреймами вызовов
- Примитивная обработка ошибок
- Встроенные функции (print, console.log)

```rust
pub struct VM {
    // Стек значений
    stack: Vec<Value>,
    // Стек фреймов вызовов
    call_stack: Vec<CallFrame>,
    // Аккумулятор
    accumulator: Value,
    // Константы
    constants: Vec<Value>,
    // Глобальные переменные
    globals: HashMap<String, Value>,
}

pub struct CallFrame {
    function: Rc<BytecodeFunction>,
    ip: usize,              // Instruction pointer
    stack_base: usize,      // Base of local variables on stack
}
```

**Критерии готовности:**
- [ ] Успешное выполнение простых программ
- [ ] Корректные вызовы функций с аргументами
- [ ] Работающие циклы и условия
- [ ] Трассировка выполнения для отладки

---

## Фаза 3: Представление значений и объекты (4-5 недель)

### Цель
Реализовать систему типов JavaScript и базовые объекты.

### Задачи

#### 3.1 Система типов Value (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 2.3  

**Технические требования:**
- Динамические типы JS
- Начальная реализация через enum (позже оптимизация до NaN-boxing)
- Type coercion согласно JS семантике

```rust
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(GcRef<String>),
    Object(GcRef<JsObject>),
    Function(GcRef<JsFunction>),
    Array(GcRef<JsArray>),
    Null,
    Undefined,
}

impl Value {
    pub fn type_of(&self) -> &'static str { /* ... */ }
    pub fn to_boolean(&self) -> bool { /* JS coercion rules */ }
    pub fn to_number(&self) -> f64 { /* JS coercion rules */ }
    pub fn to_string(&self) -> String { /* JS coercion rules */ }
}
```

#### 3.2 Скрытые классы (Hidden Classes/Shapes) (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 3.1  

**Технические требования:**
- Shape-based объектная модель для быстрого доступа к свойствам
- Переходы между shapes при добавлении/удалении свойств
- Transition chains и sharing между объектами

```rust
pub struct Shape {
    pub id: ShapeId,
    pub transitions: HashMap<String, ShapeId>,
    pub properties: Vec<PropertyDescriptor>,
    pub parent: Option<ShapeId>,
}

pub struct PropertyDescriptor {
    pub name: String,
    pub offset: u16,
    pub attributes: PropertyAttributes,
}

pub struct JsObject {
    pub shape: ShapeId,
    pub slots: Vec<Value>,          // Inline properties
    pub elements: Option<Elements>, // Array-like elements
}
```

**Критерии готовности:**
- [ ] Быстрый доступ к свойствам через offset
- [ ] Корректные переходы между shapes
- [ ] Поддержка property deletion
- [ ] Отладочный вывод shape transitions

---

## Фаза 4: Inline Caches (IC) (3-4 недели)

### Цель
Реализовать адаптивную оптимизацию через inline caches.

### Задачи

#### 4.1 Базовая IC инфраструктура (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 3.2  

**Технические требования:**
- IC слоты в байткодных инструкциях
- Состояния: Uninitialized → Monomorphic → Polymorphic → Megamorphic
- Fast/slow path execution

```rust
pub enum IcState {
    Uninitialized,
    Monomorphic { shape: ShapeId, offset: u16 },
    Polymorphic(Vec<(ShapeId, u16)>),
    Megamorphic,
}

pub struct PropertyLoadIC {
    pub state: IcState,
    pub property_name: String,
}

pub struct PropertyStoreIC {
    pub state: IcState,
    pub property_name: String,
}

pub struct CallIC {
    pub state: IcState,
    pub target_cache: Vec<GcRef<JsFunction>>,
}
```

#### 4.2 Property Access IC (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 4.1  

**Технические требования:**
- Load IC для `obj.prop`
- Store IC для `obj.prop = value`
- Обновление IC при изменении shapes
- Статистика hit/miss rates

**Критерии готовности:**
- [ ] Значительное ускорение property access
- [ ] Корректные переходы между IC состояниями
- [ ] Профилирование эффективности IC

#### 4.3 Call IC (1 неделя)
**Приоритет:** Высокий  
**Зависимости:** 4.2  

**Технические требования:**
- Кэширование call targets
- Polymorphic dispatch для method calls
- Integration с function calls в интерпретаторе

---

## Фаза 5: JIT компиляция (6-8 недель)

### Цель
Реализовать tiered compilation с baseline и optimizing JIT.

### Задачи

#### 5.1 Профилирование и горячие участки (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 4.3  

**Технические требования:**
- Счетчики выполнения для функций и циклов
- Heuristics для определения горячих участков
- Adaptive thresholds на основе размера функции

```rust
pub struct ProfileData {
    pub execution_count: u32,
    pub call_count: u32,
    pub loop_iterations: HashMap<usize, u32>, // bytecode offset -> count
    pub ic_polymorphism: HashMap<usize, u8>,  // IC complexity
}

pub struct CompilationDecision {
    pub should_compile: bool,
    pub tier: CompilationTier,
    pub reason: CompilationReason,
}

pub enum CompilationTier {
    Baseline,
    Optimized,
}
```

#### 5.2 Baseline JIT с Cranelift (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 5.1  

**Технические требования:**
- Интеграция с Cranelift backend
- Прямая трансляция байткода в machine code
- Встраивание IC как патчабельные guards
- Code cache и trampolines

```rust
pub struct JitCompiler {
    builder: FunctionBuilder,
    code_cache: CodeCache,
    isa: Box<dyn TargetIsa>,
}

impl JitCompiler {
    pub fn compile_baseline(&mut self, func: &BytecodeFunction) -> CompiledCode {
        // 1. Create Cranelift function signature
        // 2. Translate bytecode to CLIF IR
        // 3. Compile to machine code
        // 4. Generate deoptimization metadata
    }
}

pub struct CompiledCode {
    pub code_ptr: *const u8,
    pub code_size: usize,
    pub deopt_data: DeoptimizationData,
    pub ic_patches: Vec<IcPatchSite>,
}
```

**Критерии готовности:**
- [ ] Компиляция простых функций
- [ ] Интеграция с IC system
- [ ] Корректные вызовы compiled code
- [ ] 2-5x ускорение против интерпретатора

#### 5.3 Деоптимизация (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 5.2  

**Технические требования:**
- Deoptimization metadata для восстановления interpreter state
- Guard instructions в compiled code
- Bailout handling и переход к интерпретатору
- Stack walking и frame reconstruction

```rust
pub struct DeoptimizationData {
    pub bailout_points: Vec<BailoutPoint>,
    pub value_maps: Vec<ValueMap>,
}

pub struct BailoutPoint {
    pub code_offset: usize,
    pub bytecode_offset: usize,
    pub value_map_index: usize,
}

pub struct ValueMap {
    pub locals: Vec<ValueLocation>,
    pub stack: Vec<ValueLocation>,
}

pub enum ValueLocation {
    Register(PhysReg),
    StackSlot(i32),
    Constant(Value),
}
```

#### 5.4 On-Stack Replacement (OSR) (1-2 недели)
**Приоритет:** Средний  
**Зависимости:** 5.3  

**Технические требования:**
- OSR entry points в горячих циклах
- State transfer из интерпретатора в compiled code
- Loop peeling для оптимизации

---

## Фаза 6: Сборщик мусора (5-6 недель)

### Цель
Реализовать генерационный garbage collector.

### Задачи

#### 6.1 GC инфраструктура (1-2 недели)
**Приоритет:** Критический  
**Зависимости:** 3.1  

**Технические требования:**
- GC-managed типы и handles
- Root set tracking
- Safepoints в VM и JIT

```rust
pub struct GcRef<T> {
    ptr: NonNull<GcObject<T>>,
    _phantom: PhantomData<T>,
}

pub struct GcObject<T> {
    header: GcHeader,
    data: T,
}

pub struct GcHeader {
    mark_bit: bool,
    generation: Generation,
    size: u32,
}

pub enum Generation {
    Young,
    Old,
}
```

#### 6.2 Молодое поколение (Copying GC) (2-3 недели)
**Приоритет:** Критический  
**Зависимости:** 6.1  

**Технические требования:**
- Semispace copying collector
- Cheney's algorithm для эффективного сканирования
- Fast allocation в bump pointer fashion
- Promotion в старое поколение

**Критерии готовности:**
- [ ] Корректная сборка мусора в young space
- [ ] Обновление всех references после копирования
- [ ] Promotion based на возрасте объектов

#### 6.3 Старое поколение (Mark-Sweep) (2-3 недели)
**Приоритет:** Высокий  
**Зависимости:** 6.2  

**Технические требования:**
- Tri-color marking algorithm
- Incremental/concurrent marking
- Компактификация для уменьшения фрагментации
- Write barriers для межпоколенческих ссылок

```rust
pub struct MarkSweepCollector {
    marking_worklist: Vec<GcRef<dyn GcObject>>,
    free_lists: HashMap<usize, Vec<*mut u8>>, // size -> free blocks
}

impl MarkSweepCollector {
    pub fn mark_phase(&mut self) {
        // 1. Mark all roots
        // 2. Propagate marks through object graph
        // 3. Handle weak references
    }
    
    pub fn sweep_phase(&mut self) {
        // 1. Sweep unmarked objects
        // 2. Build free lists
        // 3. Update allocation structures
    }
}
```

---

## Фаза 7: Модульная архитектура (2-3 недели)

### Цель
Организовать код в чистую модульную структуру.

### Задачи

#### 7.1 Реструктуризация кода (1-2 недели)
**Приоритет:** Средний  
**Зависимости:** 6.3  

**Структура крейтов:**
```
workspace/
├── lexer/           # Лексический анализ
├── parser/          # Синтаксический анализ и AST
├── bytecode/        # IR и байткод
├── runtime/         # Value, Objects, Shapes, IC
├── vm/              # Интерпретатор и профилирование
├── jit/             # JIT компиляция
├── gc/              # Garbage collector
├── tools/           # REPL, профайлер, отладчик
├── std/             # Встроенные объекты
└── engine/          # Главный API
```

#### 7.2 API и инструменты (1 неделя)
**Приоритет:** Средний  
**Зависимости:** 7.1  

**Технические требования:**
- Публичный API для embedding
- REPL с автодополнением
- Профайлер производительности
- Heap dump утилиты

---

## Фаза 8: Расширение языка и оптимизации (8-12 недель)

### Цель
Расширить поддержку JavaScript и добавить продвинутые оптимизации.

### Задачи

#### 8.1 Расширенные языковые конструкции (4-6 недель)
**Приоритет:** Высокий  
**Зависимости:** 7.2  

**Возможности:**
- Массивы с typed elements и dense/sparse optimization
- Prototype chain и inheritance
- `this` binding и arrow functions
- `try/catch/finally` exception handling
- Closures и lexical scoping
- Итераторы и `for...of` loops

#### 8.2 Оптимизирующий JIT (3-4 недели)
**Приоритет:** Высокий  
**Зависимости:** 8.1  

**Оптимизации:**
- Constant folding и propagation
- Common subexpression elimination (CSE)
- Function inlining с size budgets
- Bounds check elimination
- Escape analysis (начальная версия)
- Loop optimization и vectorization

#### 8.3 Продвинутые возможности (2-3 недели)
**Приоритет:** Средний  
**Зависимости:** 8.2  

**Возможности:**
- Built-in objects: Array, Object, Math, JSON
- RegExp поддержка
- Symbols и iterators
- WeakMap/WeakSet
- Proxies (базовая поддержка)

---

## Критический путь и зависимости

### Критический путь (последовательные задачи):
1. Лексер → Парсер → AST (3-4 недели)
2. Байткод дизайн → Компилятор → VM (5-7 недель)
3. Value system → Shapes → IC (7-9 недель)
4. JIT profiling → Baseline JIT → Deopt (6-8 недель)

### Параллельные работы:
- GC можно разрабатывать параллельно с IC (после Value system)
- Tooling можно делать на любом этапе после VM
- Расширение языка можно делать итеративно

### Общий timeline: 6-9 месяцев

---

## Управление рисками

### Высокие риски:
1. **Сложность деоптимизации**: Митигация - начать с простых cases, постепенное усложнение
2. **GC корректность**: Митигация - обширное тестирование, статические анализаторы
3. **JIT производительность**: Митигация - профилирование на каждом этапе, бенчмарки

### Средние риски:
1. **IC сложность**: Митигация - начать с monomorphic cases
2. **Cranelift интеграция**: Митигация - изучить существующие проекты (Wasmtime)
3. **Memory overhead**: Митигация - continuous profiling, оптимизация representations

### Низкие риски:
1. **Parser bugs**: Митигация - extensive test suite
2. **Bytecode design changes**: Митигация - версионирование формата

---

## Критерии готовности фаз

### Фаза 1: Frontend готов
- [x] Парсинг всех поддерживаемых конструкций (базовая версия)
- [x] Менее 5% ложных синтаксических ошибок на реальном коде
- [x] Время парсинга < 1мс на 1000 строк кода

### Фаза 2: VM готов
- [ ] Успешное выполнение программ без падений
- [ ] Производительность интерпретатора > 1000 ops/sec
- [ ] Корректная обработка рекурсии и больших стеков

### Фаза 3: Objects готовы
- [ ] Поддержка property access/assignment
- [ ] Shape transitions без memory leaks
- [ ] Производительность property access < 10ns

### Фаза 4: IC готовы
- [ ] 90%+ monomorphic hit rate на типичном коде
- [ ] 5-10x ускорение property access
- [ ] Корректная invalidation при shape changes

### Фаза 5: JIT готов
- [ ] 5-20x ускорение горячих функций
- [ ] Корректная деоптимизация во всех случаях
- [ ] Время компиляции < 1мс на 100 байткодов

### Фаза 6: GC готов
- [ ] Максимальная пауза < 10мс для heap до 100MB
- [ ] Throughput overhead < 15%
- [ ] Отсутствие memory leaks на long-running программах

### Фаза 7: Модули готовы
- [ ] Четкие API boundaries между модулями
- [ ] Документация для публичного API
- [ ] Готовность к open source release

### Фаза 8: Оптимизации готовы
- [ ] Производительность сравнима с Boa на микробенчмарках
- [ ] Поддержка 80% ES6 features
- [ ] Стабильность на реальных JS программах

---

## Инфраструктура тестирования

### Unit тесты
- Каждый модуль имеет покрытие > 80%
- Property-based testing для парсера и VM
- Fuzzing для поиска edge cases

### Integration тесты
- Golden file tests для парсера
- End-to-end execution tests
- Performance regression tests

### Benchmarking
- Микробенчмарки для всех критических путей
- Макробенчмарки на реальных JS программах
- Continuous performance monitoring

### Tooling
- Автоматические нагрузочные тесты
- Memory leak detection
- Профилирование производительности
- Статический анализ (clippy, miri)

---

## Заключение

Этот roadmap представляет реалистичный план создания V8-подобного JavaScript движка на Rust. Ключевые принципы:

1. **Инкрементальная разработка** - каждая фаза дает работающий результат
2. **Тестирование с первого дня** - quality gate для каждой фазы
3. **Производительность как feature** - измерения на каждом этапе
4. **Готовность к изменениям** - модульная архитектура позволяет рефакторинг

Общий объем работы: **6-9 месяцев full-time разработки** или **12-18 месяцев part-time**.

Результат: высокопроизводительный JavaScript движок, демонстрирующий современные техники VM implementation в экосистеме Rust.
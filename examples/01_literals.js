// ============================================================================
// 01_literals.js - Демонстрация литералов и типов данных
// ============================================================================
// Этот пример демонстрирует все поддерживаемые типы литералов в движке

// ЧИСЛОВЫЕ ЛИТЕРАЛЫ
// =================

// Целые числа
let integer = 42;
let negativeInt = -17;
let zero = 0;

// Десятичные числа  
let decimal = 3.14159;
let fraction = 0.5;
let negativeDecimal = -2.718;

// Экспоненциальная запись (если поддерживается)
let scientific1 = 1e5;    // 100000
let scientific2 = 2.5e-3; // 0.0025

// СТРОКОВЫЕ ЛИТЕРАЛЫ  
// ===================

let singleQuoted = 'Hello, World!';
let doubleQuoted = "JavaScript Engine";
let emptyString = "";
let stringWithNumbers = "Version 1.0";

// Строки с специальными символами
let withSpaces = "Hello World";
let withPunctuation = "Hello, World! How are you?";

// БУЛЕВЫ ЛИТЕРАЛЫ
// ================

let isTrue = true;
let isFalse = false;
let boolExpression = true;

// NULL И UNDEFINED
// ================

let nullValue = null;
let undefinedValue = undefined;

// ДЕМОНСТРАЦИЯ РАБОТЫ С ЛИТЕРАЛАМИ
// =================================

// Простые операции с числами
let sum = integer + decimal;           // 42 + 3.14159 = 45.14159
let difference = integer - negativeInt; // 42 - (-17) = 59
let product = fraction * 100;          // 0.5 * 100 = 50

// Работа со строками (конкатенация через +)
let greeting = "Hello" + ", " + "World!";
let mixed = "Number: " + integer;

// Логические операции
let andResult = isTrue && isFalse;     // true && false = false
let orResult = isTrue || isFalse;      // true || false = true  
let notResult = !isTrue;               // !true = false

// Сравнение литералов
let numComparison = integer > decimal;  // 42 > 3.14159 = true
let stringComparison = "abc" < "def";   // true (лексикографическое сравнение)
let equalityCheck = integer == 42;     // true
let strictEquality = integer === 42;   // true

// РЕЗУЛЬТАТ ВЫПОЛНЕНИЯ
// ====================

// Возвращаем результат последней операции для демонстрации
strictEquality
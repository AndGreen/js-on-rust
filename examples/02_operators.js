// ============================================================================
// 02_operators.js - Демонстрация всех поддерживаемых операторов
// ============================================================================
// Этот пример демонстрирует все виды операторов, поддерживаемые движком

// АРИФМЕТИЧЕСКИЕ ОПЕРАТОРЫ
// ========================

let a = 15;
let b = 4;

// Базовые арифметические операции
let addition = a + b;        // 15 + 4 = 19
let subtraction = a - b;     // 15 - 4 = 11  
let multiplication = a * b;  // 15 * 4 = 60
let division = a / b;        // 15 / 4 = 3.75
let modulo = a % b;          // 15 % 4 = 3

// Возведение в степень
let power = 2 ** 3;          // 2^3 = 8

// Унарные операторы
let positive = +a;           // +15 = 15
let negative = -a;           // -15 = -15

// ОПЕРАТОРЫ ПРИСВАИВАНИЯ
// =====================

let x = 10;

// Составные операторы присваивания
x += 5;     // x = x + 5 = 15
x -= 3;     // x = x - 3 = 12
x *= 2;     // x = x * 2 = 24
x /= 4;     // x = x / 4 = 6
x %= 5;     // x = x % 5 = 1

// ОПЕРАТОРЫ СРАВНЕНИЯ
// ===================

let num1 = 10;
let num2 = 20;
let str1 = "10";

// Нестрогое сравнение
let equal = num1 == 10;           // true
let notEqual = num1 != num2;      // true

// Строгое сравнение
let strictEqual = num1 === 10;    // true
let strictNotEqual = num1 !== str1; // true

// Операторы отношения
let less = num1 < num2;           // 10 < 20 = true
let greater = num2 > num1;        // 20 > 10 = true
let lessEqual = num1 <= 10;       // 10 <= 10 = true
let greaterEqual = num2 >= 20;    // 20 >= 20 = true

// ЛОГИЧЕСКИЕ ОПЕРАТОРЫ
// ===================

let isTrue = true;
let isFalse = false;

// Логические операции
let logicalAnd = isTrue && isFalse;   // true && false = false
let logicalOr = isTrue || isFalse;    // true || false = true
let logicalNot = !isTrue;             // !true = false

// Сложные логические выражения
let complex1 = (num1 > 5) && (num2 < 25);    // true && true = true
let complex2 = (num1 == 0) || (num2 > 15);   // false || true = true

// БИТОВЫЕ ОПЕРАТОРЫ
// =================

let bit1 = 12;  // 1100 в двоичном
let bit2 = 10;  // 1010 в двоичном

// Битовые операции
let bitwiseAnd = bit1 & bit2;     // 1100 & 1010 = 1000 = 8
let bitwiseOr = bit1 | bit2;      // 1100 | 1010 = 1110 = 14
let bitwiseXor = bit1 ^ bit2;     // 1100 ^ 1010 = 0110 = 6
let bitwiseNot = ~bit1;           // ~1100 = ...0011 (инверсия)

// Битовые сдвиги
let leftShift = bit1 << 2;        // 1100 << 2 = 110000 = 48
let rightShift = bit1 >> 2;       // 1100 >> 2 = 11 = 3
let unsignedRightShift = bit1 >>> 2; // 1100 >>> 2 = 11 = 3

// ОПЕРАТОРЫ ИНКРЕМЕНТА И ДЕКРЕМЕНТА
// =================================

let counter = 5;

// Постфиксные операторы (пока поддерживаются только они)
let postIncrement = counter++;    // counter становится 6, возвращает 5
let postDecrement = counter--;    // counter становится 5, возвращает 6

// СПЕЦИАЛЬНЫЕ ОПЕРАТОРЫ
// =====================

// Оператор typeof (если поддерживается)
let typeOfNumber = typeof 42;
let typeOfString = typeof "hello";
let typeOfBoolean = typeof true;

// ПРИОРИТЕТ ОПЕРАТОРОВ
// ===================

// Демонстрация правильного приоритета операторов
let priority1 = 2 + 3 * 4;        // 2 + (3 * 4) = 2 + 12 = 14
let priority2 = (2 + 3) * 4;      // (2 + 3) * 4 = 5 * 4 = 20
let priority3 = 2 * 3 + 4 * 5;    // (2 * 3) + (4 * 5) = 6 + 20 = 26

// Логический приоритет
let priority4 = true || false && false;  // true || (false && false) = true

// СЛОЖНЫЕ ВЫРАЖЕНИЯ
// =================

let result1 = (a + b) * (a - b);  // (15 + 4) * (15 - 4) = 19 * 11 = 209
let result2 = a > b && b > 0 || a < 0; // (15 > 4) && (4 > 0) || (15 < 0) = true

// Условный (тернарный) оператор - если поддерживается
// let conditional = a > b ? "a больше" : "b больше или равно";

// РЕЗУЛЬТАТ ВЫПОЛНЕНИЯ
// ====================

// Возвращаем результат сложного выражения для демонстрации
result2
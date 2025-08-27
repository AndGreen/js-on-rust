// ============================================================================
// 03_functions.js - Демонстрация функций и их возможностей
// ============================================================================
// Этот пример демонстрирует объявление, вызов и различные виды функций

// ОБЪЯВЛЕНИЕ ФУНКЦИЙ
// ==================

// Простая функция без параметров
function sayHello() {
    return "Hello, World!";
}

// Функция с параметрами
function add(a, b) {
    return a + b;
}

// Функция с несколькими операциями
function calculateArea(width, height) {
    let area = width * height;
    let perimeter = 2 * (width + height);
    return area;
}

// Функция с условной логикой
function max(a, b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

// Функция с циклом
function factorial(n) {
    let result = 1;
    let i = 1;
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    return result;
}

// ВЫЗОВ ФУНКЦИЙ
// =============

// Простой вызов
let greeting = sayHello();

// Вызов с аргументами
let sum = add(5, 3);
let area = calculateArea(4, 6);
let maximum = max(10, 15);

// Вызов в выражениях
let complexCalculation = add(10, 20) * 2;
let nestedCall = max(add(5, 3), add(2, 7));

// РЕКУРСИВНЫЕ ФУНКЦИИ
// ===================

// Рекурсивный факториал
function factorialRecursive(n) {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorialRecursive(n - 1);
    }
}

// Рекурсивное вычисление числа Фибоначчи
function fibonacci(n) {
    if (n <= 1) {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

// ФУНКЦИИ С ЛОКАЛЬНЫМИ ПЕРЕМЕННЫМИ
// ================================

function processNumbers(x, y) {
    // Локальные переменные
    let temp1 = x * 2;
    let temp2 = y * 3;
    let intermediate = temp1 + temp2;
    
    // Условная обработка
    if (intermediate > 50) {
        let bonus = 10;
        return intermediate + bonus;
    } else {
        let penalty = 5;
        return intermediate - penalty;
    }
}

// ФУНКЦИОНАЛЬНЫЕ ВЫРАЖЕНИЯ
// ========================

// Анонимная функция, присвоенная переменной
let multiply = function(a, b) {
    return a * b;
};

// Использование функционального выражения
let product = multiply(4, 7);

// ПЕРЕДАЧА ФУНКЦИЙ КАК ЗНАЧЕНИЙ
// =============================

function operate(a, b, operation) {
    return operation(a, b);
}

// Использование функции как аргумента
let addResult = operate(5, 3, add);
let multiplyResult = operate(5, 3, multiply);

// ДЕМОНСТРАЦИЯ ОБЛАСТИ ВИДИМОСТИ
// ==============================

let globalVar = 100;

function testScope(param) {
    let localVar = 200;
    
    // Функция может использовать глобальные переменные
    let withGlobal = param + globalVar;
    
    // И локальные переменные
    let withLocal = param + localVar;
    
    return withGlobal + withLocal;
}

// СЛОЖНЫЕ ПРИМЕРЫ
// ===============

// Функция для вычисления суммы цифр числа
function digitSum(number) {
    let sum = 0;
    let remaining = number;
    
    while (remaining > 0) {
        let digit = remaining % 10;
        sum = sum + digit;
        remaining = (remaining - digit) / 10;
    }
    
    return sum;
}

// Функция для проверки простого числа
function isPrime(number) {
    if (number <= 1) {
        return false;
    }
    
    let i = 2;
    while (i * i <= number) {
        if (number % i == 0) {
            return false;
        }
        i = i + 1;
    }
    
    return true;
}

// ВЫПОЛНЕНИЕ И ДЕМОНСТРАЦИЯ
// =========================

// Тестирование различных функций
let fact5 = factorial(5);           // 120
let fib7 = fibonacci(7);            // 13
let processed = processNumbers(5, 8); // зависит от логики
let scope = testScope(50);          // использует глобальные и локальные переменные
let digits = digitSum(12345);       // 1+2+3+4+5 = 15
let prime = isPrime(17);            // true

// Результат для демонстрации
fact5
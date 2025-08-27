// ============================================================================
// 05_control_flow.js - Демонстрация управления потоком выполнения
// ============================================================================
// Этот пример демонстрирует все конструкции управления потоком

// УСЛОВНЫЕ КОНСТРУКЦИИ
// ====================

// Простая if конструкция
let x = 10;
if (x > 5) {
    x = x + 1;
}

// If-else конструкция
let y = 3;
if (y > 10) {
    y = y * 2;
} else {
    y = y + 5;
}

// Вложенные условия
let score = 85;
let grade;
if (score >= 90) {
    grade = "A";
} else {
    if (score >= 80) {
        grade = "B";
    } else {
        if (score >= 70) {
            grade = "C";
        } else {
            grade = "F";
        }
    }
}

// Множественные условия с логическими операторами
let age = 25;
let hasLicense = true;
let canDrive = false;

if (age >= 18 && hasLicense) {
    canDrive = true;
} else {
    canDrive = false;
}

// ЦИКЛЫ WHILE
// ===========

// Простой цикл while
let counter = 0;
let sum = 0;
while (counter < 5) {
    sum = sum + counter;
    counter = counter + 1;
}

// While с более сложными условиями
let number = 16;
let steps = 0;
while (number > 1) {
    if (number % 2 == 0) {
        number = number / 2;
    } else {
        number = number * 3 + 1;
    }
    steps = steps + 1;
}

// Вложенные циклы while
let i = 1;
let product = 1;
while (i <= 3) {
    let j = 1;
    while (j <= 2) {
        product = product * (i + j);
        j = j + 1;
    }
    i = i + 1;
}

// ЦИКЛЫ FOR
// =========

// Классический цикл for
let factorial = 1;
for (let n = 1; n <= 5; n = n + 1) {
    factorial = factorial * n;
}

// For с различными выражениями
let powers = 1;
for (let base = 2; base <= 8; base = base * 2) {
    powers = powers + base;
}

// Вложенные циклы for
let matrix_sum = 0;
for (let row = 1; row <= 3; row = row + 1) {
    for (let col = 1; col <= 3; col = col + 1) {
        matrix_sum = matrix_sum + (row * col);
    }
}

// ОПЕРАТОРЫ BREAK И CONTINUE
// ==========================

// Поиск первого четного числа больше 10
let found = false;
let result = 0;
for (let k = 11; k <= 20; k = k + 1) {
    if (k % 2 != 0) {
        continue; // пропускаем нечетные числа
    }
    result = k;
    break; // выходим при первом найденном
}

// Break в while цикле
let limit = 100;
let value = 1;
while (true) {
    value = value * 2;
    if (value > limit) {
        break;
    }
}

// ФУНКЦИИ С RETURN
// ================

// Функция с ранним возвратом
function findMax(a, b, c) {
    if (a >= b && a >= c) {
        return a;
    }
    if (b >= a && b >= c) {
        return b;
    }
    return c;
}

// Функция с множественными точками выхода
function classifyNumber(num) {
    if (num < 0) {
        return "negative";
    }
    if (num == 0) {
        return "zero";
    }
    if (num > 100) {
        return "large";
    }
    return "positive";
}

// Рекурсивная функция с условным возвратом
function gcd(a, b) {
    if (b == 0) {
        return a;
    }
    return gcd(b, a % b);
}

// СЛОЖНЫЕ КОМБИНАЦИИ
// ==================

// Функция для проверки простого числа
function isPrime(n) {
    if (n <= 1) {
        return false;
    }
    if (n <= 3) {
        return true;
    }
    if (n % 2 == 0 || n % 3 == 0) {
        return false;
    }
    
    let divisor = 5;
    while (divisor * divisor <= n) {
        if (n % divisor == 0 || n % (divisor + 2) == 0) {
            return false;
        }
        divisor = divisor + 6;
    }
    return true;
}

// Функция для генерации чисел Фибоначчи до лимита
function fibonacciUpTo(limit) {
    let count = 0;
    let a = 0;
    let b = 1;
    
    while (a <= limit) {
        count = count + 1;
        let temp = a + b;
        a = b;
        b = temp;
    }
    return count;
}

// Функция сортировки пузырьком (фрагмент)
function bubbleSortStep(arr) {
    // Упрощенная версия - один проход
    let swapped = false;
    for (let i = 0; i < 4; i = i + 1) {
        if (arr[i] > arr[i + 1]) {
            let temp = arr[i];
            arr[i] = arr[i + 1];
            arr[i + 1] = temp;
            swapped = true;
        }
    }
    return swapped;
}

// БЛОКИ И ОБЛАСТЬ ВИДИМОСТИ
// =========================

// Блок с локальными переменными
{
    let blockVar = 100;
    let blockResult = blockVar * 2;
    x = x + blockResult; // изменяем внешнюю переменную
}

// Функция с блоками внутри
function processData(input) {
    let processed = 0;
    
    if (input > 0) {
        let multiplier = 2;
        processed = input * multiplier;
    } else {
        let addend = 10;
        processed = input + addend;
    }
    
    return processed;
}

// ДЕМОНСТРАЦИЯ РАБОТЫ КОНСТРУКЦИЙ
// ===============================

// Тестирование различных функций
let maxValue = findMax(15, 23, 8);      // 23
let numberType = classifyNumber(-5);    // "negative"
let gcdResult = gcd(48, 18);            // 6
let primeCheck = isPrime(17);           // true
let fibCount = fibonacciUpTo(20);       // количество чисел Фибоначчи <= 20

// Тестирование сортировки
let testArray = [5, 2, 8, 1, 9];
let wasSorted = bubbleSortStep(testArray);

// Тестирование обработки данных
let processed1 = processData(5);        // положительное число
let processed2 = processData(-3);       // отрицательное число

// Комплексный пример - поиск наибольшего общего делителя для массива
function gcdArray() {
    let numbers = [12, 18, 24];
    let result = numbers[0];
    
    for (let i = 1; i < 3; i = i + 1) {
        result = gcd(result, numbers[i]);
    }
    
    return result;
}

let arrayGcd = gcdArray();

// Результат для демонстрации
arrayGcd
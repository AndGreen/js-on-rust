// ============================================================================
// demo_calculator.js - Простой калькулятор с поддержкой различных операций
// ============================================================================
// Демонстрирует создание простого калькулятора с функциями

// БАЗОВЫЕ ОПЕРАЦИИ
// ================

function add(a, b) {
    return a + b;
}

function subtract(a, b) {
    return a - b;
}

function multiply(a, b) {
    return a * b;
}

function divide(a, b) {
    if (b == 0) {
        return "Error: Division by zero";
    }
    return a / b;
}

function power(base, exponent) {
    if (exponent == 0) {
        return 1;
    }
    
    let result = 1;
    let i = 0;
    while (i < exponent) {
        result = result * base;
        i = i + 1;
    }
    return result;
}

// ПРОДВИНУТЫЕ ОПЕРАЦИИ
// ====================

function factorial(n) {
    if (n < 0) {
        return "Error: Negative number";
    }
    if (n == 0 || n == 1) {
        return 1;
    }
    
    let result = 1;
    let i = 2;
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    return result;
}

function fibonacci(n) {
    if (n < 0) {
        return "Error: Negative number";
    }
    if (n <= 1) {
        return n;
    }
    
    let a = 0;
    let b = 1;
    let i = 2;
    
    while (i <= n) {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }
    
    return b;
}

function gcd(a, b) {
    if (a < 0) a = -a;
    if (b < 0) b = -b;
    
    while (b != 0) {
        let temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

function lcm(a, b) {
    if (a == 0 || b == 0) {
        return 0;
    }
    if (a < 0) a = -a;
    if (b < 0) b = -b;
    
    return (a * b) / gcd(a, b);
}

// ОБЪЕКТ-КАЛЬКУЛЯТОР
// ==================

let calculator = {
    memory: 0,
    
    // Базовые операции
    add: function(a, b) {
        return a + b;
    },
    
    subtract: function(a, b) {
        return a - b;
    },
    
    multiply: function(a, b) {
        return a * b;
    },
    
    divide: function(a, b) {
        if (b == 0) {
            return null; // ошибка деления на ноль
        }
        return a / b;
    },
    
    // Операции с памятью
    store: function(value) {
        this.memory = value;
        return value;
    },
    
    recall: function() {
        return this.memory;
    },
    
    clear: function() {
        this.memory = 0;
        return 0;
    },
    
    // Составные операции
    calculate: function(a, operator, b) {
        if (operator == "+") {
            return this.add(a, b);
        }
        if (operator == "-") {
            return this.subtract(a, b);
        }
        if (operator == "*") {
            return this.multiply(a, b);
        }
        if (operator == "/") {
            return this.divide(a, b);
        }
        return null; // неизвестная операция
    }
};

// ВЫЧИСЛЕНИЕ СТАТИСТИКИ
// =====================

function calculateStatistics(numbers) {
    // numbers - массив чисел [a, b, c, d, e]
    let sum = 0;
    let count = 0;
    
    // Подсчет суммы и количества
    let i = 0;
    while (i < 10) { // максимум 10 элементов
        if (i < numbers.length && numbers[i] !== null) {
            sum = sum + numbers[i];
            count = count + 1;
        }
        i = i + 1;
    }
    
    if (count == 0) {
        return {
            sum: 0,
            average: 0,
            min: 0,
            max: 0,
            count: 0
        };
    }
    
    let average = sum / count;
    
    // Поиск минимума и максимума
    let min = numbers[0];
    let max = numbers[0];
    
    i = 1;
    while (i < count) {
        if (numbers[i] < min) {
            min = numbers[i];
        }
        if (numbers[i] > max) {
            max = numbers[i];
        }
        i = i + 1;
    }
    
    return {
        sum: sum,
        average: average,
        min: min,
        max: max,
        count: count
    };
}

// РЕШЕНИЕ КВАДРАТНЫХ УРАВНЕНИЙ
// ============================

function solveQuadratic(a, b, c) {
    if (a == 0) {
        // Линейное уравнение bx + c = 0
        if (b == 0) {
            return { type: "no_solution" };
        }
        return { 
            type: "linear", 
            solution: -c / b 
        };
    }
    
    // Квадратное уравнение ax² + bx + c = 0
    let discriminant = b * b - 4 * a * c;
    
    if (discriminant < 0) {
        return { type: "no_real_solutions" };
    }
    
    if (discriminant == 0) {
        return {
            type: "one_solution",
            solution: -b / (2 * a)
        };
    }
    
    // Приближенное вычисление квадратного корня
    function sqrt(x) {
        if (x < 0) return 0;
        if (x == 0) return 0;
        if (x == 1) return 1;
        
        let guess = x / 2;
        let precision = 0.001;
        
        while (true) {
            let better = (guess + x / guess) / 2;
            if (better - guess < precision && guess - better < precision) {
                break;
            }
            guess = better;
        }
        
        return guess;
    }
    
    let sqrtD = sqrt(discriminant);
    
    return {
        type: "two_solutions",
        solution1: (-b + sqrtD) / (2 * a),
        solution2: (-b - sqrtD) / (2 * a)
    };
}

// ДЕМОНСТРАЦИЯ РАБОТЫ КАЛЬКУЛЯТОРА
// ================================

// Тестирование базовых операций
let sum1 = add(15, 25);           // 40
let product1 = multiply(6, 7);    // 42
let power1 = power(2, 5);         // 32
let fact1 = factorial(5);         // 120
let fib1 = fibonacci(10);         // 55
let gcd1 = gcd(48, 18);          // 6
let lcm1 = lcm(12, 18);          // 36

// Работа с калькулятором-объектом
calculator.store(100);
let memory = calculator.recall();
let calc1 = calculator.calculate(memory, "*", 2);  // 200
calculator.store(calc1);

// Статистика массива
let testNumbers = [10, 20, 30, 40, 50];
let stats = calculateStatistics(testNumbers);
let average = stats.average;  // 30
let maximum = stats.max;      // 50

// Решение квадратного уравнения: x² - 5x + 6 = 0
let equation = solveQuadratic(1, -5, 6);
let hasTwoSolutions = equation.type == "two_solutions";

// Комплексное вычисление - площадь треугольника по формуле Герона
function triangleArea(a, b, c) {
    let s = (a + b + c) / 2;  // полупериметр
    let underSqrt = s * (s - a) * (s - b) * (s - c);
    
    if (underSqrt <= 0) {
        return 0; // невозможный треугольник
    }
    
    // Упрощенное вычисление квадратного корня
    let area = 1;
    let precision = 0.01;
    
    while (area * area < underSqrt - precision || area * area > underSqrt + precision) {
        area = (area + underSqrt / area) / 2;
    }
    
    return area;
}

let triangleAreaResult = triangleArea(3, 4, 5);  // должно быть около 6

// Итоговый результат для демонстрации
triangleAreaResult
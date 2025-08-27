// ============================================================================
// 04_objects_arrays.js - Демонстрация объектов и массивов
// ============================================================================
// Этот пример демонстрирует создание и работу с объектами и массивами

// СОЗДАНИЕ ОБЪЕКТОВ
// =================

// Пустой объект
let emptyObject = {};

// Объект с различными типами свойств
let person = {
    name: "John",
    age: 30,
    isActive: true,
    score: 85.5
};

// Объект с вычисляемыми свойствами
let coordinates = {
    x: 10,
    y: 20,
    z: 5
};

// Объект с числовыми ключами
let grades = {
    1: "A",
    2: "B",
    3: "C"
};

// Вложенные объекты
let company = {
    name: "TechCorp",
    employees: 100,
    address: {
        street: "Main St",
        number: 123,
        city: "Springfield"
    }
};

// ДОСТУП К СВОЙСТВАМ ОБЪЕКТОВ
// ===========================

// Точечная нотация
let personName = person.name;        // "John"
let personAge = person.age;          // 30

// Скобочная нотация
let personScore = person["score"];   // 85.5
let firstGrade = grades[1];          // "A"

// Доступ к вложенным свойствам
let companyName = company.name;      // "TechCorp"
let streetName = company.address.street; // "Main St"
let streetNumber = company.address["number"]; // 123

// ИЗМЕНЕНИЕ СВОЙСТВ ОБЪЕКТОВ
// ==========================

// Изменение существующих свойств
person.age = 31;
person["score"] = 90.0;

// Добавление новых свойств
person.email = "john@example.com";
person["phone"] = "123-456-7890";

// Изменение вложенных свойств
company.address.number = 456;

// СОЗДАНИЕ МАССИВОВ
// =================

// Пустой массив
let emptyArray = [];

// Массив чисел
let numbers = [1, 2, 3, 4, 5];

// Массив строк
let colors = ["red", "green", "blue"];

// Массив смешанных типов
let mixed = [1, "hello", true, 3.14];

// Массив с пропусками (разреженный массив)
let sparse = [1, , 3, , 5];

// Вложенные массивы (двумерный массив)
let matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];

// ДОСТУП К ЭЛЕМЕНТАМ МАССИВОВ
// ===========================

// Индексы начинаются с 0
let firstNumber = numbers[0];    // 1
let thirdColor = colors[2];      // "blue"
let lastNumber = numbers[4];     // 5

// Доступ к вложенным элементам
let matrixElement = matrix[1][2]; // 6 (вторая строка, третий столбец)

// ИЗМЕНЕНИЕ ЭЛЕМЕНТОВ МАССИВОВ
// ============================

// Изменение существующих элементов
numbers[2] = 10;        // [1, 2, 10, 4, 5]
colors[0] = "yellow";   // ["yellow", "green", "blue"]

// Добавление элементов (если поддерживается)
numbers[5] = 6;         // [1, 2, 10, 4, 5, 6]

// ОБЪЕКТЫ С МАССИВАМИ
// ===================

let student = {
    name: "Alice",
    grades: [85, 92, 78, 96],
    subjects: ["Math", "Science", "History", "English"]
};

// Доступ к элементам массива внутри объекта
let firstGrade = student.grades[0];      // 85
let secondSubject = student.subjects[1]; // "Science"

// Изменение элементов
student.grades[2] = 88;
student.subjects[3] = "Literature";

// МАССИВЫ ОБЪЕКТОВ
// ================

let employees = [
    {
        name: "Bob",
        position: "Developer",
        salary: 75000
    },
    {
        name: "Carol",
        position: "Designer", 
        salary: 65000
    },
    {
        name: "Dave",
        position: "Manager",
        salary: 85000
    }
];

// Доступ к объектам в массиве
let firstEmployee = employees[0];
let firstEmployeeName = employees[0].name;     // "Bob"
let secondEmployeePosition = employees[1].position; // "Designer"

// ФУНКЦИИ ДЛЯ РАБОТЫ С ОБЪЕКТАМИ И МАССИВАМИ
// ==========================================

// Функция для вычисления суммы элементов массива
function sumArray(arr) {
    let sum = 0;
    let i = 0;
    while (i < 5) { // предполагаем фиксированную длину
        if (i < arr.length) {
            sum = sum + arr[i];
        }
        i = i + 1;
    }
    return sum;
}

// Функция для поиска максимального элемента в массиве
function maxInArray(arr) {
    let max = arr[0];
    let i = 1;
    while (i < 5) { // предполагаем фиксированную длину
        if (i < arr.length && arr[i] > max) {
            max = arr[i];
        }
        i = i + 1;
    }
    return max;
}

// Функция для подсчета свойств объекта
function countProperties(obj) {
    // Упрощенная версия - считаем известные свойства
    let count = 0;
    if (obj.name !== undefined) count = count + 1;
    if (obj.age !== undefined) count = count + 1;
    if (obj.score !== undefined) count = count + 1;
    if (obj.isActive !== undefined) count = count + 1;
    if (obj.email !== undefined) count = count + 1;
    return count;
}

// ИСПОЛЬЗОВАНИЕ this В ОБЪЕКТАХ
// =============================

let calculator = {
    value: 0,
    add: function(n) {
        this.value = this.value + n;
        return this.value;
    },
    multiply: function(n) {
        this.value = this.value * n;
        return this.value;
    },
    reset: function() {
        this.value = 0;
        return this.value;
    }
};

// ДЕМОНСТРАЦИЯ РАБОТЫ
// ===================

// Работа с массивами
let numbersSum = sumArray(numbers);     // сумма элементов [1, 2, 10, 4, 5]
let maxNumber = maxInArray(numbers);    // максимальный элемент

// Работа с объектами
let personProps = countProperties(person); // количество свойств

// Использование калькулятора
calculator.add(10);      // value = 10
calculator.multiply(3);  // value = 30
let calcResult = calculator.value;

// Комплексная операция - вычисление средней зарплаты
let totalSalary = employees[0].salary + employees[1].salary + employees[2].salary;
let averageSalary = totalSalary / 3;

// Результат для демонстрации
averageSalary
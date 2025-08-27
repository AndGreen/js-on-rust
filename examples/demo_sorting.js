// ============================================================================
// demo_sorting.js - Демонстрация различных алгоритмов сортировки
// ============================================================================
// Показывает реализацию и сравнение алгоритмов сортировки

// УТИЛИТАРНЫЕ ФУНКЦИИ
// ===================

function swap(arr, i, j) {
    let temp = arr[i];
    arr[i] = arr[j];
    arr[j] = temp;
}

function copyArray(source, target) {
    let i = 0;
    while (i < source.length) {
        target[i] = source[i];
        i = i + 1;
    }
}

function printArray(arr, name) {
    // Упрощенный вывод - возвращаем первые несколько элементов
    let result = name + ": [";
    let i = 0;
    while (i < arr.length && i < 5) {
        result = result + arr[i];
        if (i < arr.length - 1 && i < 4) {
            result = result + ", ";
        }
        i = i + 1;
    }
    result = result + "]";
    return result;
}

// ПУЗЫРЬКОВАЯ СОРТИРОВКА
// ======================

function bubbleSort(arr) {
    let n = arr.length;
    let swapped = true;
    
    while (swapped) {
        swapped = false;
        let i = 1;
        while (i < n) {
            if (arr[i - 1] > arr[i]) {
                swap(arr, i - 1, i);
                swapped = true;
            }
            i = i + 1;
        }
        n = n - 1; // последний элемент уже на месте
    }
    
    return arr;
}

// СОРТИРОВКА ВЫБОРОМ
// ==================

function selectionSort(arr) {
    let n = arr.length;
    
    let i = 0;
    while (i < n - 1) {
        let minIndex = i;
        let j = i + 1;
        
        // Находим минимальный элемент
        while (j < n) {
            if (arr[j] < arr[minIndex]) {
                minIndex = j;
            }
            j = j + 1;
        }
        
        // Меняем местами если нужно
        if (minIndex != i) {
            swap(arr, i, minIndex);
        }
        
        i = i + 1;
    }
    
    return arr;
}

// СОРТИРОВКА ВСТАВКАМИ
// ====================

function insertionSort(arr) {
    let n = arr.length;
    let i = 1;
    
    while (i < n) {
        let key = arr[i];
        let j = i - 1;
        
        // Сдвигаем элементы больше key на одну позицию вперед
        while (j >= 0 && arr[j] > key) {
            arr[j + 1] = arr[j];
            j = j - 1;
        }
        
        arr[j + 1] = key;
        i = i + 1;
    }
    
    return arr;
}

// БЫСТРАЯ СОРТИРОВКА
// ==================

function partition(arr, low, high) {
    let pivot = arr[high];
    let i = low - 1;
    
    let j = low;
    while (j < high) {
        if (arr[j] <= pivot) {
            i = i + 1;
            swap(arr, i, j);
        }
        j = j + 1;
    }
    
    swap(arr, i + 1, high);
    return i + 1;
}

function quickSortRecursive(arr, low, high) {
    if (low < high) {
        let pi = partition(arr, low, high);
        quickSortRecursive(arr, low, pi - 1);
        quickSortRecursive(arr, pi + 1, high);
    }
}

function quickSort(arr) {
    quickSortRecursive(arr, 0, arr.length - 1);
    return arr;
}

// СОРТИРОВКА СЛИЯНИЕМ
// ===================

function merge(arr, left, middle, right) {
    let leftSize = middle - left + 1;
    let rightSize = right - middle;
    
    // Создаем временные массивы
    let leftArr = [null, null, null, null, null, null, null, null];
    let rightArr = [null, null, null, null, null, null, null, null];
    
    // Копируем данные во временные массивы
    let i = 0;
    while (i < leftSize) {
        leftArr[i] = arr[left + i];
        i = i + 1;
    }
    
    let j = 0;
    while (j < rightSize) {
        rightArr[j] = arr[middle + 1 + j];
        j = j + 1;
    }
    
    // Слияние временных массивов обратно в arr
    i = 0;
    j = 0;
    let k = left;
    
    while (i < leftSize && j < rightSize) {
        if (leftArr[i] <= rightArr[j]) {
            arr[k] = leftArr[i];
            i = i + 1;
        } else {
            arr[k] = rightArr[j];
            j = j + 1;
        }
        k = k + 1;
    }
    
    // Копируем оставшиеся элементы leftArr
    while (i < leftSize) {
        arr[k] = leftArr[i];
        i = i + 1;
        k = k + 1;
    }
    
    // Копируем оставшиеся элементы rightArr
    while (j < rightSize) {
        arr[k] = rightArr[j];
        j = j + 1;
        k = k + 1;
    }
}

function mergeSortRecursive(arr, left, right) {
    if (left < right) {
        let middle = left + (right - left) / 2;
        middle = middle - (middle % 1); // округление вниз
        
        mergeSortRecursive(arr, left, middle);
        mergeSortRecursive(arr, middle + 1, right);
        merge(arr, left, middle, right);
    }
}

function mergeSort(arr) {
    mergeSortRecursive(arr, 0, arr.length - 1);
    return arr;
}

// ПРОВЕРКА СОРТИРОВАННОСТИ
// ========================

function isSorted(arr) {
    let i = 1;
    while (i < arr.length) {
        if (arr[i - 1] > arr[i]) {
            return false;
        }
        i = i + 1;
    }
    return true;
}

// ИЗМЕРЕНИЕ ПРОИЗВОДИТЕЛЬНОСТИ (УПРОЩЕННОЕ)
// ========================================

function measureSort(sortFunction, arr, name) {
    let testArr = [null, null, null, null, null, null, null, null];
    copyArray(arr, testArr);
    
    let startTime = 0; // упрощение - нет реального времени
    let result = sortFunction(testArr);
    let endTime = 100; // упрощение
    
    return {
        name: name,
        sorted: isSorted(result),
        array: result,
        time: endTime - startTime
    };
}

// СПЕЦИАЛИЗИРОВАННЫЕ АЛГОРИТМЫ
// ============================

// Подсчетная сортировка для небольших целых чисел
function countingSort(arr, maxValue) {
    let count = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // до 10
    let output = [null, null, null, null, null, null, null, null];
    
    // Подсчет вхождений
    let i = 0;
    while (i < arr.length) {
        if (arr[i] >= 0 && arr[i] < 10) {
            count[arr[i]] = count[arr[i]] + 1;
        }
        i = i + 1;
    }
    
    // Кумулятивные суммы
    i = 1;
    while (i < 10) {
        count[i] = count[i] + count[i - 1];
        i = i + 1;
    }
    
    // Построение результата
    i = arr.length - 1;
    while (i >= 0) {
        if (arr[i] >= 0 && arr[i] < 10) {
            let pos = count[arr[i]] - 1;
            output[pos] = arr[i];
            count[arr[i]] = count[arr[i]] - 1;
        }
        i = i - 1;
    }
    
    // Копирование результата
    i = 0;
    while (i < arr.length) {
        arr[i] = output[i];
        i = i + 1;
    }
    
    return arr;
}

// Сортировка по возрастанию/убыванию
function sortDescending(arr) {
    bubbleSort(arr); // сначала по возрастанию
    
    // Разворачиваем массив
    let left = 0;
    let right = arr.length - 1;
    
    while (left < right) {
        swap(arr, left, right);
        left = left + 1;
        right = right - 1;
    }
    
    return arr;
}

// ТЕСТИРОВАНИЕ ВСЕХ АЛГОРИТМОВ
// ============================

// Исходный массив для тестирования
let originalArray = [64, 34, 25, 12, 22, 11, 90, 5];

// Создаем копии для каждого алгоритма
let bubbleArray = [64, 34, 25, 12, 22, 11, 90, 5];
let selectionArray = [64, 34, 25, 12, 22, 11, 90, 5];
let insertionArray = [64, 34, 25, 12, 22, 11, 90, 5];
let quickArray = [64, 34, 25, 12, 22, 11, 90, 5];
let mergeArray = [64, 34, 25, 12, 22, 11, 90, 5];
let countingArray = [6, 3, 2, 1, 2, 1, 9, 0]; // для подсчетной сортировки
let descendingArray = [64, 34, 25, 12, 22, 11, 90, 5];

// Тестирование алгоритмов
let bubbleResult = measureSort(bubbleSort, bubbleArray, "Bubble Sort");
let selectionResult = measureSort(selectionSort, selectionArray, "Selection Sort");
let insertionResult = measureSort(insertionSort, insertionArray, "Insertion Sort");
let quickResult = measureSort(quickSort, quickArray, "Quick Sort");
let mergeResult = measureSort(mergeSort, mergeArray, "Merge Sort");

// Специальные случаи
let countingResult = countingSort(countingArray, 10);
let descendingResult = sortDescending(descendingArray);

// Проверка результатов
let allSorted = bubbleResult.sorted && selectionResult.sorted && 
                insertionResult.sorted && quickResult.sorted && 
                mergeResult.sorted;

// Сравнение первых элементов отсортированных массивов
let firstElements = bubbleResult.array[0] + selectionResult.array[0] + 
                   insertionResult.array[0] + quickResult.array[0] + 
                   mergeResult.array[0];

// Результат - сумма первых элементов всех отсортированных массивов
firstElements
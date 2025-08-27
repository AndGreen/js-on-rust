// ============================================================================
// 06_advanced.js - Продвинутые примеры комбинирования всех возможностей
// ============================================================================
// Этот пример демонстрирует сложные алгоритмы и структуры данных

// АЛГОРИТМ БЫСТРОЙ СОРТИРОВКИ (УПРОЩЕННАЯ ВЕРСИЯ)
// ===============================================

function quickSortPartition(arr, low, high) {
    let pivot = arr[high];
    let i = low - 1;
    
    for (let j = low; j < high; j = j + 1) {
        if (arr[j] <= pivot) {
            i = i + 1;
            let temp = arr[i];
            arr[i] = arr[j];
            arr[j] = temp;
        }
    }
    
    let temp = arr[i + 1];
    arr[i + 1] = arr[high];
    arr[high] = temp;
    
    return i + 1;
}

function quickSort(arr, low, high) {
    if (low < high) {
        let pi = quickSortPartition(arr, low, high);
        quickSort(arr, low, pi - 1);
        quickSort(arr, pi + 1, high);
    }
    return arr;
}

// РЕАЛИЗАЦИЯ СТЕКА
// ================

let stack = {
    items: [null, null, null, null, null], // фиксированный размер
    top: -1,
    maxSize: 5,
    
    push: function(item) {
        if (this.top < this.maxSize - 1) {
            this.top = this.top + 1;
            this.items[this.top] = item;
            return true;
        }
        return false; // переполнение стека
    },
    
    pop: function() {
        if (this.top >= 0) {
            let item = this.items[this.top];
            this.items[this.top] = null;
            this.top = this.top - 1;
            return item;
        }
        return null; // стек пуст
    },
    
    peek: function() {
        if (this.top >= 0) {
            return this.items[this.top];
        }
        return null;
    },
    
    isEmpty: function() {
        return this.top == -1;
    },
    
    size: function() {
        return this.top + 1;
    }
};

// РЕАЛИЗАЦИЯ ОЧЕРЕДИ
// ==================

let queue = {
    items: [null, null, null, null, null], // фиксированный размер
    front: 0,
    rear: -1,
    count: 0,
    maxSize: 5,
    
    enqueue: function(item) {
        if (this.count < this.maxSize) {
            this.rear = (this.rear + 1) % this.maxSize;
            this.items[this.rear] = item;
            this.count = this.count + 1;
            return true;
        }
        return false; // очередь переполнена
    },
    
    dequeue: function() {
        if (this.count > 0) {
            let item = this.items[this.front];
            this.items[this.front] = null;
            this.front = (this.front + 1) % this.maxSize;
            this.count = this.count - 1;
            return item;
        }
        return null; // очередь пуста
    },
    
    isEmpty: function() {
        return this.count == 0;
    },
    
    size: function() {
        return this.count;
    }
};

// АЛГОРИТМ ПОИСКА В ГЛУБИНУ (DFS) ДЛЯ ГРАФА
// =========================================

let graph = {
    // Представление графа списком смежности (упрощенное)
    adj: {
        0: [1, 2],
        1: [0, 3, 4], 
        2: [0, 5],
        3: [1],
        4: [1, 5],
        5: [2, 4]
    },
    
    dfs: function(start, target) {
        let visited = [false, false, false, false, false, false];
        let path = [];
        
        function dfsHelper(node) {
            visited[node] = true;
            path.push(node);
            
            if (node == target) {
                return true;
            }
            
            let neighbors = graph.adj[node];
            for (let i = 0; i < neighbors.length; i = i + 1) {
                let neighbor = neighbors[i];
                if (!visited[neighbor]) {
                    if (dfsHelper(neighbor)) {
                        return true;
                    }
                }
            }
            
            path.pop(); // backtrack
            return false;
        }
        
        let found = dfsHelper(start);
        return { found: found, path: path };
    }
};

// СИСТЕМА УПРАВЛЕНИЯ ПАМЯТЬЮ (УПРОЩЕННАЯ)
// =======================================

let memoryManager = {
    heap: [null, null, null, null, null, null, null, null], // 8 блоков
    freeList: [0, 1, 2, 3, 4, 5, 6, 7], // свободные блоки
    freeCount: 8,
    
    allocate: function(size) {
        if (this.freeCount >= size) {
            let allocated = [];
            for (let i = 0; i < size; i = i + 1) {
                let block = this.freeList[this.freeCount - 1];
                allocated.push(block);
                this.freeCount = this.freeCount - 1;
            }
            return allocated;
        }
        return null; // не хватает памяти
    },
    
    deallocate: function(blocks) {
        for (let i = 0; i < blocks.length; i = i + 1) {
            let block = blocks[i];
            this.heap[block] = null;
            this.freeList[this.freeCount] = block;
            this.freeCount = this.freeCount + 1;
        }
    },
    
    getFragmentation: function() {
        return (8 - this.freeCount) / 8; // процент занятой памяти
    }
};

// ПАТТЕРН НАБЛЮДАТЕЛЬ (OBSERVER)
// ==============================

let eventSystem = {
    listeners: {},
    
    on: function(event, callback) {
        if (!this.listeners[event]) {
            this.listeners[event] = [];
        }
        // Упрощенная версия - добавляем в первый свободный слот
        let callbacks = this.listeners[event];
        for (let i = 0; i < 5; i = i + 1) {
            if (!callbacks[i]) {
                callbacks[i] = callback;
                break;
            }
        }
    },
    
    emit: function(event, data) {
        let callbacks = this.listeners[event];
        if (callbacks) {
            for (let i = 0; i < 5; i = i + 1) {
                if (callbacks[i]) {
                    callbacks[i](data);
                }
            }
        }
    }
};

// КЭШИРОВАНИЕ ВЫЧИСЛЕНИЙ (МЕМОИЗАЦИЯ)
// ===================================

let fibonacciCache = {
    cache: {},
    
    compute: function(n) {
        if (n <= 1) {
            return n;
        }
        
        // Проверяем кэш (упрощенная версия)
        if (n == 5 && this.cache.n5 !== undefined) {
            return this.cache.n5;
        }
        if (n == 6 && this.cache.n6 !== undefined) {
            return this.cache.n6;
        }
        if (n == 7 && this.cache.n7 !== undefined) {
            return this.cache.n7;
        }
        
        // Вычисляем и кэшируем
        let result = this.compute(n - 1) + this.compute(n - 2);
        
        if (n == 5) this.cache.n5 = result;
        if (n == 6) this.cache.n6 = result;
        if (n == 7) this.cache.n7 = result;
        
        return result;
    }
};

// ПРОСТЕЙШИЙ ИНТЕРПРЕТАТОР ВЫРАЖЕНИЙ
// =================================

let calculator = {
    evaluate: function(expression) {
        // expression = { type: "binary", op: "+", left: 5, right: 3 }
        if (expression.type == "number") {
            return expression.value;
        }
        
        if (expression.type == "binary") {
            let left = this.evaluate(expression.left);
            let right = this.evaluate(expression.right);
            
            if (expression.op == "+") {
                return left + right;
            }
            if (expression.op == "-") {
                return left - right;
            }
            if (expression.op == "*") {
                return left * right;
            }
            if (expression.op == "/") {
                return left / right;
            }
        }
        
        return 0;
    }
};

// ДЕМОНСТРАЦИЯ ВСЕХ АЛГОРИТМОВ
// ============================

// Тестирование быстрой сортировки
let testArray = [3, 6, 8, 10, 1, 2, 1];
let sortedArray = quickSort(testArray, 0, 6);

// Работа со стеком
stack.push(10);
stack.push(20);
stack.push(30);
let stackTop = stack.pop(); // 30
let stackSize = stack.size(); // 2

// Работа с очередью
queue.enqueue(100);
queue.enqueue(200);
queue.enqueue(300);
let queueFirst = queue.dequeue(); // 100
let queueSize = queue.size(); // 2

// Поиск в графе
let searchResult = graph.dfs(0, 5);
let pathFound = searchResult.found;

// Управление памятью
let allocated = memoryManager.allocate(3);
let fragmentation = memoryManager.getFragmentation();
memoryManager.deallocate(allocated);

// Система событий
function logEvent(data) {
    let message = "Event received: " + data;
}

eventSystem.on("test", logEvent);
eventSystem.emit("test", "Hello World");

// Кэширование Фибоначчи
let fib7 = fibonacciCache.compute(7); // с кэшированием
let fib7Again = fibonacciCache.compute(7); // из кэша

// Интерпретатор выражений
let expr = {
    type: "binary",
    op: "+",
    left: { type: "number", value: 15 },
    right: { 
        type: "binary", 
        op: "*", 
        left: { type: "number", value: 3 },
        right: { type: "number", value: 4 }
    }
};
let result = calculator.evaluate(expr); // 15 + (3 * 4) = 27

// Финальный результат - комплексное вычисление
let complexResult = result + fib7 + stackSize + queueSize;
complexResult
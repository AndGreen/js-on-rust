// ============================================================================
// demo_data_structures.js - Реализация базовых структур данных
// ============================================================================
// Демонстрирует стек, очередь, связанный список и другие структуры

// РЕАЛИЗАЦИЯ СТЕКА
// ================

let Stack = {
    create: function() {
        return {
            items: [null, null, null, null, null, null, null, null], // фиксированный размер
            top: -1,
            maxSize: 8,
            
            push: function(item) {
                if (this.top < this.maxSize - 1) {
                    this.top = this.top + 1;
                    this.items[this.top] = item;
                    return true;
                }
                return false; // переполнение
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
            },
            
            clear: function() {
                this.top = -1;
                let i = 0;
                while (i < this.maxSize) {
                    this.items[i] = null;
                    i = i + 1;
                }
            }
        };
    }
};

// РЕАЛИЗАЦИЯ ОЧЕРЕДИ
// ==================

let Queue = {
    create: function() {
        return {
            items: [null, null, null, null, null, null, null, null],
            front: 0,
            rear: -1,
            count: 0,
            maxSize: 8,
            
            enqueue: function(item) {
                if (this.count < this.maxSize) {
                    this.rear = (this.rear + 1) % this.maxSize;
                    this.items[this.rear] = item;
                    this.count = this.count + 1;
                    return true;
                }
                return false; // очередь полна
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
            
            front: function() {
                if (this.count > 0) {
                    return this.items[this.front];
                }
                return null;
            },
            
            isEmpty: function() {
                return this.count == 0;
            },
            
            size: function() {
                return this.count;
            },
            
            clear: function() {
                this.front = 0;
                this.rear = -1;
                this.count = 0;
                let i = 0;
                while (i < this.maxSize) {
                    this.items[i] = null;
                    i = i + 1;
                }
            }
        };
    }
};

// РЕАЛИЗАЦИЯ СВЯЗАННОГО СПИСКА
// ============================

let LinkedList = {
    createNode: function(data) {
        return {
            data: data,
            next: null
        };
    },
    
    create: function() {
        return {
            head: null,
            size: 0,
            
            append: function(data) {
                let newNode = LinkedList.createNode(data);
                
                if (this.head == null) {
                    this.head = newNode;
                } else {
                    let current = this.head;
                    while (current.next != null) {
                        current = current.next;
                    }
                    current.next = newNode;
                }
                this.size = this.size + 1;
            },
            
            prepend: function(data) {
                let newNode = LinkedList.createNode(data);
                newNode.next = this.head;
                this.head = newNode;
                this.size = this.size + 1;
            },
            
            get: function(index) {
                if (index < 0 || index >= this.size) {
                    return null;
                }
                
                let current = this.head;
                let i = 0;
                while (i < index) {
                    current = current.next;
                    i = i + 1;
                }
                return current.data;
            },
            
            remove: function(index) {
                if (index < 0 || index >= this.size) {
                    return null;
                }
                
                if (index == 0) {
                    let data = this.head.data;
                    this.head = this.head.next;
                    this.size = this.size - 1;
                    return data;
                }
                
                let current = this.head;
                let i = 0;
                while (i < index - 1) {
                    current = current.next;
                    i = i + 1;
                }
                
                let data = current.next.data;
                current.next = current.next.next;
                this.size = this.size - 1;
                return data;
            },
            
            indexOf: function(data) {
                let current = this.head;
                let index = 0;
                
                while (current != null) {
                    if (current.data == data) {
                        return index;
                    }
                    current = current.next;
                    index = index + 1;
                }
                return -1; // не найдено
            },
            
            isEmpty: function() {
                return this.size == 0;
            },
            
            length: function() {
                return this.size;
            }
        };
    }
};

// РЕАЛИЗАЦИЯ ХЭША (УПРОЩЕННАЯ)
// ============================

let HashTable = {
    create: function(size) {
        return {
            buckets: [null, null, null, null, null, null, null, null], // фиксированный размер 8
            size: size || 8,
            
            hash: function(key) {
                let hash = 0;
                let i = 0;
                while (i < key.length) {
                    // Простая хэш-функция
                    hash = hash + key.charCodeAt(i);
                    i = i + 1;
                }
                return hash % this.size;
            },
            
            set: function(key, value) {
                let index = this.hash(key);
                
                if (this.buckets[index] == null) {
                    this.buckets[index] = [];
                }
                
                // Поиск существующего ключа
                let bucket = this.buckets[index];
                let i = 0;
                while (i < 4) { // максимум 4 элемента в bucket
                    if (bucket[i] && bucket[i].key == key) {
                        bucket[i].value = value;
                        return;
                    }
                    i = i + 1;
                }
                
                // Добавление нового элемента
                i = 0;
                while (i < 4) {
                    if (!bucket[i]) {
                        bucket[i] = { key: key, value: value };
                        return;
                    }
                    i = i + 1;
                }
            },
            
            get: function(key) {
                let index = this.hash(key);
                let bucket = this.buckets[index];
                
                if (bucket == null) {
                    return null;
                }
                
                let i = 0;
                while (i < 4) {
                    if (bucket[i] && bucket[i].key == key) {
                        return bucket[i].value;
                    }
                    i = i + 1;
                }
                
                return null; // не найдено
            },
            
            has: function(key) {
                return this.get(key) != null;
            },
            
            remove: function(key) {
                let index = this.hash(key);
                let bucket = this.buckets[index];
                
                if (bucket == null) {
                    return false;
                }
                
                let i = 0;
                while (i < 4) {
                    if (bucket[i] && bucket[i].key == key) {
                        bucket[i] = null;
                        return true;
                    }
                    i = i + 1;
                }
                
                return false;
            }
        };
    }
};

// РЕАЛИЗАЦИЯ ДЕКА (ДВУНАПРАВЛЕННАЯ ОЧЕРЕДЬ)
// =========================================

let Deque = {
    create: function() {
        return {
            items: [null, null, null, null, null, null, null, null],
            front: 0,
            rear: 0,
            count: 0,
            maxSize: 8,
            
            addFront: function(item) {
                if (this.count < this.maxSize) {
                    this.front = (this.front - 1 + this.maxSize) % this.maxSize;
                    this.items[this.front] = item;
                    this.count = this.count + 1;
                    return true;
                }
                return false;
            },
            
            addRear: function(item) {
                if (this.count < this.maxSize) {
                    this.items[this.rear] = item;
                    this.rear = (this.rear + 1) % this.maxSize;
                    this.count = this.count + 1;
                    return true;
                }
                return false;
            },
            
            removeFront: function() {
                if (this.count > 0) {
                    let item = this.items[this.front];
                    this.items[this.front] = null;
                    this.front = (this.front + 1) % this.maxSize;
                    this.count = this.count - 1;
                    return item;
                }
                return null;
            },
            
            removeRear: function() {
                if (this.count > 0) {
                    this.rear = (this.rear - 1 + this.maxSize) % this.maxSize;
                    let item = this.items[this.rear];
                    this.items[this.rear] = null;
                    this.count = this.count - 1;
                    return item;
                }
                return null;
            },
            
            isEmpty: function() {
                return this.count == 0;
            },
            
            size: function() {
                return this.count;
            }
        };
    }
};

// ДЕМОНСТРАЦИЯ РАБОТЫ СТРУКТУР ДАННЫХ
// ===================================

// Тестирование стека
let stack = Stack.create();
stack.push(10);
stack.push(20);
stack.push(30);
let stackPop1 = stack.pop();      // 30
let stackPop2 = stack.pop();      // 20
let stackSize = stack.size();     // 1

// Тестирование очереди
let queue = Queue.create();
queue.enqueue(100);
queue.enqueue(200);
queue.enqueue(300);
let queueDeq1 = queue.dequeue();  // 100
let queueDeq2 = queue.dequeue();  // 200
let queueSize = queue.size();     // 1

// Тестирование связанного списка
let list = LinkedList.create();
list.append(1);
list.append(2);
list.append(3);
list.prepend(0);
let listGet1 = list.get(0);       // 0
let listGet2 = list.get(2);       // 2
let listIndex = list.indexOf(3);  // 3
let listSize = list.length();     // 4

// Тестирование хэш-таблицы
let hash = HashTable.create(8);
hash.set("name", "John");
hash.set("age", "25");
hash.set("city", "New York");
let hashName = hash.get("name");  // "John"
let hashAge = hash.get("age");    // "25"
let hashHas = hash.has("city");   // true

// Тестирование дека
let deque = Deque.create();
deque.addRear(1);
deque.addFront(2);
deque.addRear(3);
deque.addFront(4);
let dequeFront = deque.removeFront(); // 4
let dequeRear = deque.removeRear();   // 3
let dequeSize = deque.size();         // 2

// ПРАКТИЧЕСКОЕ ПРИМЕНЕНИЕ
// =======================

// Проверка сбалансированности скобок с помощью стека
function isBalanced(expression) {
    let stack = Stack.create();
    let i = 0;
    
    while (i < expression.length) {
        let char = expression[i];
        
        if (char == "(" || char == "[" || char == "{") {
            stack.push(char);
        } else if (char == ")" || char == "]" || char == "}") {
            if (stack.isEmpty()) {
                return false;
            }
            
            let top = stack.pop();
            if ((char == ")" && top != "(") ||
                (char == "]" && top != "[") ||
                (char == "}" && top != "{")) {
                return false;
            }
        }
        
        i = i + 1;
    }
    
    return stack.isEmpty();
}

// Реверс строки с помощью стека
function reverseString(str) {
    let stack = Stack.create();
    let i = 0;
    
    // Заполняем стек
    while (i < str.length) {
        stack.push(str[i]);
        i = i + 1;
    }
    
    // Извлекаем символы
    let result = "";
    while (!stack.isEmpty()) {
        result = result + stack.pop();
    }
    
    return result;
}

// Тестирование практических примеров
let balanced1 = isBalanced("()[]{}");     // true
let balanced2 = isBalanced("([)]");       // false
let reversed = reverseString("hello");    // "olleh"

// Итоговый результат - сумма размеров всех структур
let totalSize = stackSize + queueSize + listSize + dequeSize;
totalSize
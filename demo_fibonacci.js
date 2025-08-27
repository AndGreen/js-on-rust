// Демонстрация последовательности Фибоначчи с for-циклом
let n = 8;
let a = 0;
let b = 1;

if (n <= 1) {
    n;
} else {
    for (let i = 2; i <= n; i = i + 1) {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b;
}
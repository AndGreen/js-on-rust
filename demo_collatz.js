// Демонстрация гипотезы Коллатца (3n+1)
let n = 7;
let steps = 0;

while (n != 1) {
    if (n % 2 == 0) {
        n = n / 2;
    } else {
        n = n * 3 + 1;
    }
    steps = steps + 1;
}

steps
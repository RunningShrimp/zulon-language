int add(int a, int b) {
    return a + b;
}

int main() {
    int result = 0;
    for (int i = 0; i < 100000; i++) {
        result = add(result, i);
    }
    return result;
}

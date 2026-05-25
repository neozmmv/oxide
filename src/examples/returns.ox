
float add(float a, float b) {
    return a + b;
}

string greet(string name) {
    return "Hello, ${name}!";
}

// return types with (?) can return null.

string? findUser(int id) {
    if (id == 1) return "Bob";
    return null; // ok
}
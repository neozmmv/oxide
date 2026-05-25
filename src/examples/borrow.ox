// like rust, we free the memory when the value gets out of scope
// functions take ownership of values, we lend the value with &

void main() {
    nameToGreet := "Bob";
    greet(&nameToGreet); // pass with &, since we are lending the value
    // nameToGreet still exists here
    takeOwnershipAndGreet(nameToGreet);
    // value dropped on function end
}

void greet(&string name) {
    printf("Hello, ${name}!\n");
    return;
}

void takeOwnershipAndGreet(string name) {
    printf("Hello, ${name}!\n");
    return;
}

// parameter value modifying inside functions
// passing reference to a value
void increment(&int value) {
    value++;
}

void increaseByTwo(&const int value) {
    value += 2;
    // ERROR!
    // &const cannot be reassigned
}

/*
    EXAMPLE:
    i := 0;
    increment(i);
    // i now evaluates to 1.
*/
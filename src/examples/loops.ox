// never type means the function never reaches to the end, never returns. Similar to TypeScript.
// or throws an error

never main() {
    while (true) {
        println("Hello forever!");
    }
}

// other example, with void main

void main() {
    repeatLoop := 10;

    for (i := 0; i < repeatLoop; i++) {
        println(i);
    }

    // or with rust-like reassigned

    for(int i = 0 .. repeatLoop) {
        println(i); // 0 to 9
    }

    for(int i = 0 ..= repeatLoop) {
        println(i); // 0 to 10
    }
}
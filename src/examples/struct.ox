struct Person {
    int age;
    string name;
}

// &Person since we are modifying a value
void (&Person p) setAge(int age) {
    p.age = age;
}

int (Person p) getAge() {
    return p.age;
}
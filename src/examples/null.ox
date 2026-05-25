void main() {
    string name = null;
    println(name) // null;

    length := name?.length; // null if name is null

    string display = name ?? "Display Name";
}
void main() {
    const int value = 5;

    if (value > 10) {
        println("value greater than 10!");
    } else if (value == 5) {
        println("value is exactly 5.");
    } else {
        println("value is something else");
    }

    const bool option = true;

    if (value == 5 && option) {
        println("value equals 5 and option is true");
    } else if (value == 5 || option) {
        println("either value equals 5 or option is true, or both");
    } else if (!option) {
        println("option is false!");
    }
}
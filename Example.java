public final class Example {

    public static int fib(int x) {
        if (x <= 1) {
            return x;
        }
        return fib(x - 1) + fib(x - 2);
    }

    public static void main(String[] args) {
        System.out.println(Integer.toString(fib(30)));
    }
}

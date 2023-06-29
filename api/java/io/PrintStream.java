package java.io;

public class PrintStream extends OutputStream {

    public native void write(String s);

    public PrintStream(OutputStream out) {
    }

    public void print(boolean b) {
        write(b ? "true" : "false");
    }

    public void print(String s) {
        if (s == null) {
            write("null");
        }
        write(s);
    }

    public void println(String s) {
        if (s == null) {
            write("null");
        }
        write(s);
        write("\n");
    }
}

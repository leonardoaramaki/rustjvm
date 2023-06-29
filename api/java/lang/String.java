package java.lang;

public final class String {

    private char value[];
    private int count;
    private int offset;

    public String() {
        this.value = new char[0];
        this.offset = 0;
    }

    public String(char[] value) {
        this.offset = 0;
        this.count = value.length;
        this.value = new char[this.count];
        for (int i = 0; i < this.value.length; i++) {
            this.value[i] = value[i];
        }
    }

    public String(char[] value, int s, int e) {
        this.offset = 0;
        this.count = e - s + 1;
        this.value = new char[this.count];
        for (int i = s, j = 0; i < e; i++, j++) {
            this.value[j] = value[i];
        }
    }

    String(char[] value, boolean share) {
        this.offset = 0;
        this.value = value;
    }

    String(int offset, int count, char value[]) {
        this.value = value;
        this.offset = offset;
        this.count = count;
    }

    public char charAt(int index) {
        return value[index];
    }
    
    public int length() {
        return this.count;
    }

    public void getChars(int srcBegin, int srcEnd, char[] dst, int dstBegin) {

    }

    public static String valueOf(int i) {
        return Integer.toString(i);
    }

    public static String valueOf(boolean b) {
        return Boolean.toString(b);
    }

    public String toString() {
        return this;
    }
}

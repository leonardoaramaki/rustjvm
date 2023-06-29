package java.lang;

public final class StringBuilder {

    private char[] value;
    private int count;

    public StringBuilder() {
        value = new char[16];
    }

    public StringBuilder(String s) {
        this.value = new char[20];
        //TODO: check capacity
        int len = s.length();
        for (int i = 0; i < len; i++) {
            this.value[i] = s.charAt(i);
        }
        count += len;
    }

    public StringBuilder append(String s) {
        //TODO: check capacity
        int len = s.length();
        for (int i = count, j = 0; j < len; i++, j++) {
            this.value[i] = s.charAt(j);
        }
        count += len;
        return this;
    }

    public StringBuilder append(int v) {
        return this;
    }

    public int length() {
        return count;
    }
    
    public String toString() {
        return new String(value, 0, this.count);
    }
}

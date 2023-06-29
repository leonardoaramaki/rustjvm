package java.lang;

import java.io.OutputStream;
import java.io.PrintStream;

/**
 * @author Leonardo Aramaki
 */
public final class System {

    public static final PrintStream out = new PrintStream(new OutputStream());

    private System() {
    }

    public static native final long currentTimeMillis();
}
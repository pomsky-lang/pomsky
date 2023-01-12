import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

public class Tester {
    public static void main(String[] args) {
        String regex = args[1];
        try {
            Pattern.compile(regex);
        } catch (PatternSyntaxException e) {
            System.err.println(e.getMessage());
            System.exit(1);
        }
    }
}

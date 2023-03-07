import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

public class Tester {
    public static void main(String[] args) {
        String regex = args[1];
        try {
            var pattern = Pattern.compile("(?U)" + regex);
            for (int i = 2; i < args.length; i++) {
                var matcher = pattern.matcher(args[i]);
                if (matcher.matches()) {
                    var region = matcher.regionStart() + ".." + matcher.regionEnd();
                    System.err.println("[matches in " + region + "] " + args[i]);
                } else {
                    System.err.println("[no match] " + args[i]);
                }
            }
        } catch (PatternSyntaxException e) {
            System.err.println(e.getMessage());
            System.exit(1);
        }
    }
}

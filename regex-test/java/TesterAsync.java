import java.util.Scanner;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

public class TesterAsync {
    public static void main(String[] args) {
        try (Scanner input = new Scanner(System.in)) {
            while (input.hasNext()) {
                String regex = input.nextLine();
                if (!regex.startsWith("REGEX:")) {
                    continue;
                }

                try {
                    Pattern p = Pattern.compile("(?U)" + regex.substring(6));
                    System.out.printf("success\n");

                    while (input.hasNext()) {
                        String line = input.nextLine();
                        if (line.startsWith("TEST:")) {
                            String test = line.substring(5);
                            if (p.matcher(test).matches()) {
                                System.out.printf("test good\n");
                            } else {
                                System.out.printf("Regex '%s' does not match '%s'\n", regex, test);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                } catch (PatternSyntaxException e) {
                    String message = e.getMessage()
                            .replaceAll("\\\\", "\\\\\\\\")
                            .replaceAll("\n", "\\\\n");
                    System.out.printf("%s\n", message);
                }
            }
        }
    }
}

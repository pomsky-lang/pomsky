import java.util.Scanner;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

public class TesterAsync {
    public static void main(String[] args) {
        try (Scanner input = new Scanner(System.in)) {
            while (input.hasNext()) {
                String regex = input.nextLine();
                try {
                    Pattern.compile(regex);
                    System.out.printf("success\n");
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

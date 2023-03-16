using System;
using System.Text.RegularExpressions;

public class TesterAsync {
    public static void Main(string[] args) {
        string line;
        while ((line = Console.ReadLine()) != null && line != "") {
            try {
                new Regex(line, RegexOptions.Compiled);
                Console.WriteLine("success");
            } catch (ArgumentException e) {
                string message = e.Message
                    .Replace(@"\\", @"\\\\")
                    .Replace("\n", @"\\n");
                Console.WriteLine("{0}", message);
            }
        }
    }
}

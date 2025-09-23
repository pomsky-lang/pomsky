using System;
using System.Text.RegularExpressions;

string line;
string testLine;

while ((line = Console.ReadLine()) != null) {
    if (!line.StartsWith("REGEX:")) {
        continue;
    }

    try {
        var r = new Regex(line.Substring(6), RegexOptions.Compiled);
        Console.WriteLine("success");

        while ((testLine = Console.ReadLine()) != null && testLine.StartsWith("TEST:")) {
            var test = testLine.Substring(5);
            if (r.IsMatch(test)) {
                Console.WriteLine("test good");
            } else {
                throw new ArgumentException($"Regex '{r}' does not match '{test}'");
            }
        }
    } catch (ArgumentException e) {
        string message = e.Message.Replace("\\", @"\\").Replace("\n", @"\n");
        Console.WriteLine(message);
    }
}

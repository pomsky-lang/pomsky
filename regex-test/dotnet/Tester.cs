using System;
using System.Text.RegularExpressions;

class Tester {
    public static void Main(string[] args) {
        try {
            Regex r = new Regex(args[1], RegexOptions.Compiled);
            for (int i = 2; i < args.Length; i++) {
                MatchCollection matches = r.Matches(args[i]);
                if (matches.Count > 0) {
                    Match match = matches[0];
                    string region = match.Index + ".." + (match.Index + match.Value.Length);
                    Console.Error.WriteLine("[matches in {0}] {1}", region, args[i]);
                } else {
                    Console.Error.WriteLine("[no match] {0}", args[i]);
                }
            }
        } catch (ArgumentException e) {
            Console.Error.WriteLine(e.Message);
            Environment.Exit(1);
        }
    }
}

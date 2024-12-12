use helptext::{sections, Help, HelpSection};
use supports_color::Stream;

const USAGE: Help = Help(sections![
    "USAGE" {
        ["pomsky [OPTIONS] <INPUT>\n\
        pomsky [OPTIONS] --path <PATH>\n\
        command | pomsky [OPTIONS]"]
    }

    ["For more information try " c:"--help"]
]);

// Shared help message parts

const FLAVORS: &[HelpSection] = sections![
    table Compact {
        "pcre"       => { ["PCRE2 regex engines, compatible with Perl, PHP and R"] }
        "python"     => { ["Python's " c!"re" " module"] }
        "java"       => { ["Java's " c!"Pattern" " class, compatible with Kotlin and Scala"] }
        "javascript" => { ["ECMAScript regular expressions"] }
        "dotnet"     => { [c!"Regex" " class in .NET languages such C# and F#"] }
        "ruby"       => { ["Ruby's built-in regular expressions"] }
        "rust"       => { ["Rust's " c!"regex" " crate"] }
        "re2"        => { ["The RE2 engine, compatible with Go's " c!"regexp" " package"] }
    }
];

const ENGINES: &[HelpSection] = sections![
    table Compact {
        "pcre2"       => { ["PCRE2 regex engine, using the " c:"pcre" " flavor"] }
    }
];

const FEATURES: &[HelpSection] = sections! [
    table Compact {
        "ascii-mode"      => { ["Allows disabling Unicode mode globally with " g:"disable unicode;"] }
        "atomic-groups"   => { ["Allows atomic groups such as " g:"atomic('if' | 'else')"] }
        "boundaries"      => { ["Allows matching word boundaries and anchors " g:"%" ", " g:"!%" ", " g:"^" ", " g:"$"] }
        "dot"             => { ["Allows matching the dot " g:"."] }
        "grapheme"        => { ["Allows matching a grapheme cluster with " g:"Grapheme" " or " g:"G"] }
        "intersection"    => { ["Allows intersecting character sets with " g:"&"] }
        "lazy-mode"       => { ["Allows enabling lazy mode globally with " g:"enable lazy;"] }
        "lookahead"       => { ["Allows (negative) lookahead, e.g. " g:"(>> 'test')"] }
        "lookbehind"      => { ["Allows (negative) lookbehind, e.g. " g:"(<< 'test')"] }
        "named-groups"    => { ["Allows named capturing groups such as " g:":test('test')"] }
        "numbered-groups" => { ["Allows normal capturing groups such as " g:":('test')"] }
        "ranges"          => { ["Allows ranges, e.g. " g:"range '1'-'255'"]
                               [y!"warning" ": compiling ranges with many digits may be slow"] }
        "recursion"       => { ["Allows " g:"recursion"] }
        "references"      => { ["Allows referencing another capturing group, e.g. " g:"::2"] }
        "regexes"         => { ["Allows literal regular expressions, e.g. " g:"regex '[]^-]'"]
                               [y!"warning" ": does not guarantee that the output is well-formed"] }
        "variables"       => { ["Allows declaring variables, e.g. " g:"let num = ['0'-'9']+;"]
                               [y!"warning" ": compiling a lot of variables may be slow"] }
    }
];

const WARNINGS: &[HelpSection] = sections![
    Short ["Disable certain warnings (disable all with " c:"-W0" ")"]
    Long ["Disable some or all warnings. A single warning can be disabled by specifying
the name followed by " c:"=0" ", for example:

    " c!"-Wcompat=0" "

Multiple warnings can be disabled by setting this option multiple times, or
using a comma-separated list:

    " c!"-Wcompat=0 -Wdeprecated=0
    -Wcompat=0,deprecated=0" "

To disable all warnings, use " c:"-W0" ".

Currently, the following warnings can be disabled:"]
    Long table Compact {
        "compat"     => { ["Compatibility warnings"] }
        "deprecated" => { ["A used feature will be removed in the future"] }
    }
];

// Actual help messages

pub(super) const HELP: Help = Help(sections![
    [g!"pomsky " {env!("CARGO_PKG_VERSION")}]
    Long ["Home page: https://pomsky-lang.org"]
    ["\n\
    Compile pomsky expressions, a new regular expression language\n\
    \n\
    Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]

    "USAGE" {
        ["pomsky [OPTIONS] <INPUT>\n\
        pomsky [OPTIONS] --path <PATH>\n\
        command | pomsky [OPTIONS]"]
    }

    "SUBCOMMANDS" {
        table Auto {
            "pomsky test" => {
                ["Run unit tests in pomsky expressions"]
                Long ["\n\
                Run " c:"pomsky test --help" " for more information"]
            }
        }
    }

    "ARGS" {
        table Auto {
            "<INPUT>" => {
                ["Pomsky expression to compile"]
                Long ["\n\
                To learn about the pomsky language, start here:\n\
                https://pomsky-lang.org/docs/"]
            }
        }
    }

    "OPTIONS" {
        table Auto {
            "    --allowed-features <FEATURE>..." => {
                ["Comma-separated list of allowed features [default: all enabled]"]
                Long ["Supported features are listed below."]
            }
            "-f, --flavor <FLAVOR>" => {
                ["Regex flavor [default: " c:"pcre" "]"]
                Long ["Supported flavors are listed below."]
            }
            "-h, --help" => {
                ["Print help information"]
                Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
            }
            "    --list shorthands" => {
                ["Show all available character class shorthands"]
            }
            "-n, --no-new-line" => {
                ["Don't print a new-line after the output"]
            }
            "-p, --path <FILE>" => {
                ["File containing the pomsky expression to compile"]
            }
            "-V, --version" => {
                ["Print version information"]
            }
            "-W, --warnings <DIAGNOSTICS>" => WARNINGS
            "-d, --debug" => {
                Long ["Show debug information"]
            }
            "    --json" => {
                Long ["Return output as JSON"]
            }
        }
    }

    Long "FLAVORS" FLAVORS
    Long "FEATURES" FEATURES
]);

pub(super) const TEST_HELP: Help = Help(sections![
    [g!"pomsky test " {env!("CARGO_PKG_VERSION")}]
    Long ["Home page: https://pomsky-lang.org"]
    ["\n\
    Test pomsky expressions, a new regular expression language\n\
    \n\
    Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]

    "USAGE" {
        ["pomsky test [OPTIONS] --path <PATH>\n\
        pomsky test [OPTIONS] <INPUT>\n\
        command | pomsky test [OPTIONS]"]
    }

    "ARGS" {
        table Auto {
            "<INPUT>" => {
                ["Pomsky expression to test"]
                Long ["\n\
                To learn about the pomsky language, start here:\n\
                https://pomsky-lang.org/docs/"]
            }
        }
    }

    "OPTIONS" {
        table Auto {
            "    --allowed-features <FEATURE>..." => {
                ["Comma-separated list of allowed features [default: all enabled]"]
                Long ["Supported features are listed below."]
            }
            "-f, --flavor <FLAVOR>" => {
                ["Regex flavor"]
                Long ["Supported flavors are listed below."]
                Long ["If " c:"--engine" " is specified, the flavor can be omitted."]
            }
            "-e, --engine <ENGINE>" => {
                ["Regex engine used for testing"]
                Long ["If " c:"--flavor" " is specified, the engine can be omitted."]
            }
            "-h, --help" => {
                ["Print help information"]
                Long ["Use " c:"-h" " for short descriptions and " c:"--help" " for more details."]
            }
            "-p, --path <FILE>" => {
                ["File or directory containing the pomsky expressions to compile"]
                Long ["If a directory is specified, all contained " c:"*.pomsky" " files are tested.
Note that pomsky respects " c:".gitignore" " files."]
            }
            "    --pass-with-no-tests" => {
                ["Don't error if the specified directory contains no " c:"*.pomsky" " files"]
            }
            "-W, --warnings <DIAGNOSTICS>" => WARNINGS
            "    --json" => {
                Long ["Report test results as JSON"]
            }
        }
    }

    Long "FLAVORS" FLAVORS
    Long "ENGINES" ENGINES
    Long "FEATURES" FEATURES
]);

pub(crate) fn print_short_usage_and_help_err() {
    let _ = USAGE.write(&mut std::io::stderr().lock(), false, is_colored(Stream::Stderr));
}

pub(crate) fn print_short_help() {
    let _ = HELP.write(&mut std::io::stdout().lock(), false, is_colored(Stream::Stdout));
}

pub(crate) fn print_long_help() {
    let _ = HELP.write(&mut std::io::stdout().lock(), true, is_colored(Stream::Stdout));
}

pub(crate) fn print_test_short_help() {
    let _ = TEST_HELP.write(&mut std::io::stdout().lock(), false, is_colored(Stream::Stdout));
}

pub(crate) fn print_test_long_help() {
    let _ = TEST_HELP.write(&mut std::io::stdout().lock(), true, is_colored(Stream::Stdout));
}

fn is_colored(stream: Stream) -> bool {
    matches!(
        supports_color::on_cached(stream),
        Some(supports_color::ColorLevel { has_basic: true, .. })
    )
}

pub(crate) fn print_version() {
    println!(concat!("pomsky ", env!("CARGO_PKG_VERSION")));
}

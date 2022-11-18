use crate::format::Help;

const USAGE: Help = Help(sections![
    "USAGE" {
        ["pomsky [OPTIONS] <INPUT>\n\
        pomsky [OPTIONS] --path <PATH>\n\
        command | pomsky [OPTIONS]"]
    }

    ["For more information try " c:"--help"]
]);

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

    "ARGS" {
        table Auto {
            "<INPUT>" => {
                ["Pomsky expression to compile"]
                Long ["\n\
                To learn about the pomsky language, start here:\n\
                https://pomsky-lang.org/docs/language-tour/basics/"]
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
            "-n, --no-new-line" => {
                ["Don't print a new-line after the output"]
            }
            "-p, --path <FILE>" => {
                ["File containing the pomsky expression to compile"]
            }
            "-V, --version" => {
                ["Print version information"]
            }
            "-W, --warnings <DIAGNOSTICS>" => {
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
            }
            "-d, --debug" => {
                Long ["Show debug information"]
            }
        }
    }

    Long "FLAVORS" {
        table Compact {
            "pcre"       => { ["PCRE/PCRE2 regex engines, compatible with Perl, PHP and R"] }
            "python"     => { ["Python's " c!"re" " module"] }
            "java"       => { ["Java's " c!"Pattern" " class, compatible with Kotlin and Scala"] }
            "javascript" => { ["ECMAScript regular expressions"] }
            "dotnet"     => { [c!"Regex" " class in .NET languages such C# and F#"] }
            "ruby"       => { ["Ruby's built-in regular expressions"] }
            "rust"       => { ["Rust's " c!"regex" " crate"] }
        }
    }

    Long "FEATURES" {
        table Compact {
            "atomic-groups"   => { ["Allows atomic groups such as " g:"atomic('if' | 'else')"] }
            "boundaries"      => { ["Allows matching word boundaries and anchors " g:"%" ", " g:"!%" ", " g:"^" ", " g:"$"] }
            "dot"             => { ["Allows matching the dot " g:"."] }
            "grapheme"        => { ["Allows matching a grapheme cluster with " g:"Grapheme" " or " g:"G"] }
            "lazy-mode"       => { ["Allows enabling lazy mode globally with " g:"enable lazy;"] }
            "lookahead"       => { ["Allows (negative) lookahead, e.g. " g:"(>> 'test')"] }
            "lookbehind"      => { ["Allows (negative) lookbehind, e.g. " g:"(<< 'test')"] }
            "named-groups"    => { ["Allows named capturing groups such as " g:":test('test')"] }
            "numbered-groups" => { ["Allows normal capturing groups such as " g:":('test')"] }
            "ranges"          => { ["Allows ranges, e.g. " g:"range '1'-'255'"]
                                   [y!"warning" ": compiling ranges with many digits may be slow"] }
            "references"      => { ["Allows referencing another capturing group, e.g. " g:"::2"] }
            "regexes"         => { ["Allows literal regular expressions, e.g. " g:"regex '[]^-]'"]
                                   [y!"warning" ": does not guarantee that the output is well-formed"] }
            "variables"       => { ["Allows declaring variables, e.g. " g:"let num = ['0'-'9']+;"]
                                   [y!"warning" ": compiling a lot of variables may be slow"] }
        }
    }
]);

pub(crate) fn print_short_usage_and_help_err() {
    let _ = USAGE.write(
        &mut std::io::stderr().lock(),
        false,
        matches!(
            supports_color::on_cached(atty::Stream::Stderr),
            Some(supports_color::ColorLevel { has_basic: true, .. })
        ),
    );
}

pub(crate) fn print_help() {
    let _ = HELP.write(
        &mut std::io::stdout().lock(),
        false,
        matches!(
            supports_color::on_cached(atty::Stream::Stdout),
            Some(supports_color::ColorLevel { has_basic: true, .. })
        ),
    );
}

pub(crate) fn print_long_help() {
    let _ = HELP.write(
        &mut std::io::stdout().lock(),
        true,
        matches!(
            supports_color::on_cached(atty::Stream::Stdout),
            Some(supports_color::ColorLevel { has_basic: true, .. })
        ),
    );
}

pub(crate) fn print_version() {
    println!(concat!("pomsky ", env!("CARGO_PKG_VERSION")));
}

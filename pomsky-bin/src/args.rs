use std::{io::Read, path::PathBuf, str::FromStr, string::FromUtf8Error};

use atty::Stream;
use pomsky::{error::DiagnosticKind, features::PomskyFeatures, options::RegexFlavor};

pub(super) enum ParseArgsError {
    Lexopt(lexopt::Error),
    StdinUtf8(FromUtf8Error),
    UnexpectedTwice(&'static str),
    Other(String),
}

impl From<lexopt::Error> for ParseArgsError {
    fn from(e: lexopt::Error) -> Self {
        ParseArgsError::Lexopt(e)
    }
}

/// Compile a Pomsky expression to a regex
#[derive(Debug)]
pub(crate) struct Args {
    /// Pomsky expression to compile
    pub(crate) input: Input,
    /// Show debug information
    pub(crate) debug: bool,
    /// Regex flavor
    pub(crate) flavor: Option<RegexFlavor>,
    /// Does not print a new-line at the end of the compiled regular expression
    pub(crate) no_new_line: bool,
    /// Set of allowed pomsky features
    pub(crate) allowed_features: PomskyFeatures,
    /// Set of allowed pomsky features
    pub(crate) warnings: DiagnosticSet,
}

#[derive(Debug)]
pub(crate) enum DiagnosticSet {
    All,
    Disabled(Vec<DiagnosticKind>),
    Enabled(Vec<DiagnosticKind>),
}

impl DiagnosticSet {
    pub(crate) fn is_enabled(&self, kind: DiagnosticKind) -> bool {
        match self {
            DiagnosticSet::All => true,
            DiagnosticSet::Disabled(set) => !set.contains(&kind),
            DiagnosticSet::Enabled(set) => set.contains(&kind),
        }
    }
}

/// Compile a Pomsky expression to a regex
#[derive(Debug)]
pub(crate) enum Input {
    Value(String),
    File(PathBuf),
}

pub(super) fn print_short_usage_and_help_err() {
    efprintln!(
        "\
%y.USAGE.%:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]

For more information try %c.--help.%",
    )
}

fn print_help() {
    fprintln!(lit concat!("\
%g.pomsky.% ", env!("CARGO_PKG_VERSION"), "
Home page: https://pomsky-lang.org

Compile pomsky expressions, a new regular expression language

Use %c.-h.% for short descriptions and %c.--help.% for more details.

%y.USAGE:.%
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]

%y.ARGS:.%
    %g.<INPUT>.%    Pomsky expression to compile

%y.OPTIONS:.%
        %g.--allowed-features <FEATURE>....%
            Comma-separated list of allowed features [default: all enabled]
    %g.-f, --flavor <FLAVOR>.%                Regex flavor [default: %c.pcre.%]
    %g.-h, --help.%                           Print help information
    %g.-n, --no-new-line.%                    Don't print a new-line after the output
    %g.-p, --path <FILE>.%                    File containing the pomsky expression to compile
    %g.-V, --version.%                        Print version information
    %g.-W, --warnings <DIAGNOSTICS>.%         Disable certain warnings (disable all with %c.-W0.%)"
    ));
}

fn print_long_help() {
    fprintln!(lit concat!("\
%g.pomsky.% ", env!("CARGO_PKG_VERSION"), "
Compile pomsky expressions, a new regular expression language

Use %c.-h.% for short descriptions and %c.--help.% for more details.

%y.USAGE:.%
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]

%y.ARGS:.%
    %g.<INPUT>.%
            Pomsky expression to compile.

            To learn about the pomsky language, start here:
            https://pomsky-lang.org/docs/language-tour/basics/

%y.OPTIONS:.%
        %g.--allowed-features <FEATURE>....%
            Comma-separated list of allowed features [default: all enabled]
            Supported features are listed below.

    %g.-f, --flavor <FLAVOR>.%
            Regex flavor [default: %c.pcre.%]
            Supported flavors are listed below.

    %g.-h, --help.%
            Print help information
            Use %c.-h.% for short descriptions and %c.--help.% for more details.

    %g.-n, --no-new-line.%
            Don't print a new-line after the output

    %g.-p, --path <FILE>.%
            File containing the pomsky expression to compile

    %g.-V, --version.%
            Print version information

    %g.-W, --warnings <DIAGNOSTICS>.%
            Disable some or all warnings. A single warning can be disabled by
            specifying the name followed by `%c.=0.%`, for example:

                %c.-Wcompat=0.%
            
            Multiple warnings can be disabled by setting this option multiple
            times, or using a comma-separated list:

                %c.-Wcompat=0 -Wdeprecated=0.%
                %c.-Wcompat=0,deprecated=0.%
            
            To disable all warnings, use %c.-W0.%

            Currently, the following warnings can be disabled:
                compat          Compatibility warnings
                deprecated      A used feature will be removed in the future

    %g.-d, --debug.%
            Show debug information

%y.FLAVORS:.%
    pcre          PCRE/PCRE2 regex engines, compatible with Perl, PHP and R
    python        Python's %g.re.% module
    java          Java's %g.Pattern.% class, compatible with Kotlin and Scala
    javascript    ECMAScript regular expressions
    dotnet        %g.Regex.% class in .NET languages such C# and F#
    ruby          Ruby's built-in regular expressions
    rust          Rust's %g.regex.% crate

%y.FEATURES:.%
    atomic-groups     Allows atomic groups such as %g.atomic('if' | 'else').%
    boundaries        Allows matching word boundaries and anchors (%g.%.%, %g.!%.%, %g.^.%, %g.$.%)
    dot               Allows matching the dot (%g...%)
    grapheme          Allows matching a grapheme cluster with %g.Grapheme.% or %g.G.%
    lazy-mode         Allows enabling lazy mode globally with %g.enable lazy;.%
    lookahead         Allows (negative) lookahead, e.g. %g.(>> 'test').%
    lookbehind        Allows (negative) lookbehind, e.g. %g.(<< 'test').%
    named-groups      Allows named capturing groups such as %g.:test('test').%
    numbered-groups   Allows normal capturing groups such as %g.:('test').%
    ranges            Allows ranges, e.g. %g.range '1'-'255'.%
                      %y.warning.%: compiling ranges with many digits may be slow
    references        Allows referencing another capturing group, e.g. %g.::2.%
    regexes           Allows literal regular expressions, e.g. %g.regex '[]^-]'.%
                      %y.warning.%: does not guarantee that the output is well-formed
    variables         Allows declaring variables, e.g. %g.let num = ['0'-'9']+;.%
                      %y.warning.%: compiling a lot of variables may be slow
"
    ));
}

fn print_version() {
    fprintln!(lit concat!("pomsky ", env!("CARGO_PKG_VERSION")));
}

pub(super) fn parse_args() -> Result<Args, ParseArgsError> {
    use lexopt::prelude::*;

    let mut arg_count = 0;
    let mut input_value = None;
    let mut path = None;
    let mut debug = false;
    let mut flavor = None;
    let mut no_new_line = false;
    let mut allowed_features = None;
    let mut warnings = DiagnosticSet::All;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        arg_count += 1;

        match arg {
            Short('p') | Long("path") => {
                if path.is_some() {
                    return Err(ParseArgsError::UnexpectedTwice("--path"));
                }
                path = Some(parser.value()?.parse()?);
            }
            Short('d') | Long("debug") => {
                if debug {
                    return Err(ParseArgsError::UnexpectedTwice("--debug"));
                }
                debug = true;
            }
            Short('f') | Long("flavor") => {
                if flavor.is_some() {
                    return Err(ParseArgsError::UnexpectedTwice("--flavor"));
                }
                let value = parser.value()?;
                let lower = value.to_string_lossy().to_ascii_lowercase();
                flavor = Some(match lower.as_str() {
                    "pcre" => RegexFlavor::Pcre,
                    "python" => RegexFlavor::Python,
                    "java" => RegexFlavor::Java,
                    "js" | "javascript" => RegexFlavor::JavaScript,
                    "dotnet" | ".net" => RegexFlavor::DotNet,
                    "ruby" => RegexFlavor::Ruby,
                    "rust" => RegexFlavor::Rust,
                    _ => {
                        return Err(ParseArgsError::Other(format!(
                            "{value:?} isn't a valid value for '--flavor <FLAVOR>'\n\
                            [possible values: pcre, python, java, javascript, dotnet, ruby, rust]"
                        )))
                    }
                });
            }
            Short('n') | Long("no-new-line") => {
                if no_new_line {
                    return Err(ParseArgsError::UnexpectedTwice("no-new-line"));
                }
                no_new_line = true;
            }
            Short('W') | Long("warnings") => {
                let value = parser.value()?;
                let value = value.to_string_lossy();
                if value.as_ref() == "0" {
                    warnings = DiagnosticSet::Enabled(vec![]);
                } else {
                    let mut warning_list_own = vec![];
                    let warning_list = match &mut warnings {
                        DiagnosticSet::Disabled(set) => set,
                        DiagnosticSet::Enabled(_) => continue,
                        DiagnosticSet::All => &mut warning_list_own,
                    };

                    for warning in value.split(',') {
                        let (kind_str, val) =
                            warning.trim_start().rsplit_once('=').ok_or_else(|| {
                                ParseArgsError::Other(format!(
                                    "`{warning}` contains no `=`, try `-W{warning}=0` \
                                    to disable {warning} warnings"
                                ))
                            })?;

                        if val != "0" {
                            return Err(ParseArgsError::Other(format!(
                                "warnings can only be disabled, try `-W{kind_str}=0`"
                            )));
                        }

                        let kind = DiagnosticKind::from_str(kind_str).map_err(|_| {
                            ParseArgsError::Other(format!(
                                "`{kind_str}` is not a recognized diagnostic kind"
                            ))
                        })?;

                        let (DiagnosticKind::Compat | DiagnosticKind::Deprecated) = kind else {
                            return Err(ParseArgsError::Other(format!(
                                "`{kind_str}` diagnostic kind cannot be disabled"
                            )))
                        };

                        warning_list.push(kind);
                    }

                    if matches!(warnings, DiagnosticSet::All) {
                        warnings = DiagnosticSet::Disabled(warning_list_own);
                    }
                }
            }
            Long("allowed-features") => {
                if allowed_features.is_some() {
                    return Err(ParseArgsError::UnexpectedTwice("--allowed-features"));
                }
                let value = parser.value()?;
                let lower = value.to_string_lossy().to_ascii_lowercase();

                let mut features = PomskyFeatures::new();
                for part in lower.split(',') {
                    let part = part.trim();
                    if !part.is_empty() {
                        match part {
                            "grapheme" => features.grapheme(true),
                            "numbered-groups" => features.numbered_groups(true),
                            "named-groups" => features.named_groups(true),
                            "atomic-groups" => features.atomic_groups(true),
                            "references" => features.references(true),
                            "lazy-mode" => features.lazy_mode(true),
                            "ranges" => features.ranges(true),
                            "variables" => features.variables(true),
                            "lookahead" => features.lookahead(true),
                            "lookbehind" => features.lookbehind(true),
                            "boundaries" => features.boundaries(true),
                            "regexes" => features.regexes(true),
                            "dot" => features.dot(true),
                            s => {
                                efprintln!("%Y.warning.%: unknown feature `{s}`");
                                features
                            }
                        };
                    }
                }
                allowed_features = Some(features);
            }
            Value(val) if input_value.is_none() => {
                input_value = Some(val.into_string().map_err(lexopt::Error::from)?);
            }
            Short('h') => {
                print_help();
                std::process::exit(0);
            }
            Long("help") => {
                print_long_help();
                std::process::exit(0);
            }
            Short('V') | Long("version") => {
                print_version();
                std::process::exit(0);
            }
            _ => Err(arg.unexpected())?,
        }
    }

    if arg_count == 0 {
        print_help();
        std::process::exit(0);
    }

    let input = match (input_value, path) {
        (Some(input), None) => Input::Value(input),
        (None, Some(path)) => Input::File(path),
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => Input::Value(input),
                Err(e) => return Err(ParseArgsError::StdinUtf8(e)),
            }
        }
        (Some(_), Some(_)) => {
            return Err(ParseArgsError::Other(
                "You can only provide an input or a path, but not both".into(),
            ))
        }
        (None, None) => return Err(ParseArgsError::Other("No input provided".into())),
    };

    Ok(Args {
        input,
        flavor,
        debug,
        no_new_line,
        allowed_features: allowed_features.unwrap_or_default(),
        warnings,
    })
}

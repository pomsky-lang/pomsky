// these tests don't work on Windows for some reason
#![cfg(not(target_os = "windows"))]

use assert_cmd::prelude::*;
use assert_fs::prelude::{FileWriteBin, FileWriteStr};
use predicates::reflection::{Case, Parameter, PredicateReflection};

use pomsky::diagnose::DiagnosticCode;
use pomsky_bin::{CompilationResult, Diagnostic, Kind, Severity, Span, Timings, Version};

use std::{fmt, process::Command};

pub struct Output {
    ignore_visual: bool,
    expected: CompilationResult,
}

impl Output {
    pub fn new(expected: CompilationResult) -> Self {
        Output { ignore_visual: true, expected }
    }

    pub fn ignore_visual(mut self, ignore_visual: bool) -> Self {
        self.ignore_visual = ignore_visual;
        self
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(&self.expected).unwrap())
    }
}

impl predicates::Predicate<[u8]> for Output {
    fn eval(&self, variable: &[u8]) -> bool {
        match serde_json::from_slice::<CompilationResult>(variable) {
            Ok(mut res) => {
                res.timings.all = 0;
                if self.ignore_visual {
                    for d in &mut res.diagnostics {
                        d.visual = String::new();
                    }
                }
                self.expected == res
            }
            Err(_) => false,
        }
    }

    fn find_case(&self, expected: bool, variable: &[u8]) -> Option<Case> {
        let actual = self.eval(variable);
        if expected == actual {
            Some(Case::new(Some(self), actual))
        } else {
            None
        }
    }
}

impl PredicateReflection for Output {
    fn parameters<'a>(&'a self) -> Box<dyn Iterator<Item = Parameter<'a>> + 'a> {
        let params = [Parameter::new("expected output", self)];
        Box::new(params.into_iter())
    }
}

const RED: &str = "\u{1b}[31m";
// const RED_BOLD: &str = "\u{1b}[31;1m";
const RESET: &str = "\u{1b}[0m";

const ERROR: &str = "error: \n  × ";
const ERROR_COLOR: &str = "\u{1b}[31;1merror\u{1b}[0m: \n  \u{1b}[31m×\u{1b}[0m ";
const USAGE: &str = r#"
USAGE:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]
For more information try `--help`
"#;
const USAGE_COLOR: &str = "\n\
\u{1b}[33mUSAGE\u{1b}[0m:\n    \
    pomsky [OPTIONS] <INPUT>\n    \
    pomsky [OPTIONS] --path <PATH>\n    \
    command | pomsky [OPTIONS]\n\
For more information try \u{1b}[36m--help\u{1b}[0m\n";

fn command(args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("pomsky").unwrap();
    for arg in args {
        cmd.arg(arg);
    }
    cmd
}

fn command_color(args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("pomsky").unwrap();
    for arg in args {
        cmd.arg(arg);
    }
    cmd.env("FORCE_COLOR", "1");
    cmd
}

#[test]
fn version() {
    let mut cmd = command(&["-V"]);
    cmd.assert().success().stderr("").stdout(format!("pomsky {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help() {
    let mut cmd = command(&["-h"]);
    cmd.assert().success().stderr("").stdout(format!(r#"pomsky {}

Compile pomsky expressions, a new regular expression language

Use `-h` for short descriptions and `--help` for more details.

USAGE:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]

ARGS:
    <INPUT>  Pomsky expression to compile

OPTIONS:
        --allowed-features <FEATURE>...  Comma-separated list of allowed features [default: all enabled]
    -f, --flavor <FLAVOR>                Regex flavor [default: `pcre`]
    -h, --help                           Print help information
    -n, --no-new-line                    Don't print a new-line after the output
    -p, --path <FILE>                    File containing the pomsky expression to compile
    -V, --version                        Print version information
    -W, --warnings <DIAGNOSTICS>         Disable certain warnings (disable all with `-W0`)
"#, env!("CARGO_PKG_VERSION")));
}

#[test]
fn unknown_flag() {
    let mut cmd = command(&["-k", "test/file/doesnt/exist"]);
    cmd.assert().failure().stderr(format!(
        "{ERROR}invalid option '-k'

USAGE:
    pomsky [OPTIONS] <INPUT>
    pomsky [OPTIONS] --path <PATH>
    command | pomsky [OPTIONS]
For more information try `--help`
"
    ));
}

#[test]
fn file_doesnt_exist() {
    let mut cmd = command(&["-p", "test/file/doesnt/exist"]);
    cmd.assert().failure().stderr(format!("{ERROR}No such file or directory (os error 2)\n"));

    let mut cmd = command_color(&["-p", "test/file/doesnt/exist"]);
    cmd.assert().failure().stderr(format!("{ERROR_COLOR}No such file or directory (os error 2)\n"));
}

#[test]
fn empty_input() {
    let mut cmd = command(&[]);
    cmd.assert().success().stdout("\n").stderr("");
}

#[test]
fn pretty_print() {
    let mut cmd = command(&[
        "let x = >> 'test'?;
        x{2} | x{3,5} | . C ![w d s n r t a e f] ['a'-'f'] | range '0'-'7F' base 16 |\
        :x() ::x | (!<< 'a')+ | regex '['",
        "--debug",
    ]);
    cmd.assert()
        .success()
        .stdout(
            "(?=(?:test)?){2}|(?=(?:test)?){3,5}|.[\\s\\S]\
            [^\\w\\d\\s\\n\\r\\t\\x07\\x1B\\f][a-f]|\
            0|[1-7][0-9a-fA-F]?|[8-9a-fA-F]|(?P<x>)\\1|(?<!a)+|[\n",
        )
        .stderr(
            r#"======================== debug ========================
let x = (>>
  "test"{0,1}
);
| x{2}
| x{3,5}
| .
  C
  ![word digit space n r t a e f]
  ['a'-'f']
| range '0'-'7F' base 16
| :x(
    ""
  )
  ::x
| (
    !<< "a"
  ){1,}
| regex "["

"#,
        );
}

#[test]
fn arg_input() {
    let mut cmd = command(&[":foo('test')+"]);
    cmd.assert().success().stdout("(?P<foo>test)+\n").stderr("");
}

#[test]
fn arg_input_with_flavor() {
    let mut cmd = command(&[":foo('test')+", "-f", "js"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");

    let mut cmd = command(&[":foo('test')+", "-fjs"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");

    let mut cmd = command(&[":foo('test')+", "-f=js"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");

    let mut cmd = command(&[":foo('test')+", "--flavor=js"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");

    let mut cmd = command(&[":foo('test')+", "--flavor", "js"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");

    let mut cmd = command(&[":foo('test')+", "-f", "jS"]);
    cmd.assert().success().stdout("(?<foo>test)+\n").stderr("");
}

#[test]
fn invalid_flavor() {
    let mut cmd = command(&[":foo('test')+", "-f", "jsx"]);
    cmd.assert().failure().stderr(format!(
        "{ERROR}'jsx' isn't a valid flavor
  │ possible values: pcre, python, java, javascript, dotnet, ruby, rust
{USAGE}"
    ));

    let mut cmd = command_color(&[":foo('test')+", "-f", "jsx"]);
    cmd.assert().failure().stderr(format!(
        "{ERROR_COLOR}'jsx' isn't a valid flavor
  {RED}│{RESET} possible values: pcre, python, java, javascript, dotnet, ruby, rust
{USAGE_COLOR}"
    ));
}

#[test]
fn flavor_used_multiple_times() {
    let mut cmd = command(&[":foo('test')+", "-fjs", "-f", "rust"]);
    cmd.assert().failure().stderr(format!(
        "{ERROR}The argument '--flavor' was provided more than once, but cannot be used
  │ multiple times
{USAGE}"
    ));
}

#[test]
fn input_and_path() {
    let mut cmd = command(&[":foo('test')+", "-p", "foo"]);
    cmd.assert().failure().stderr(format!(
        "{ERROR}You can only provide an input or a path, but not both
{USAGE}"
    ));
}

#[test]
fn path() {
    let file = assert_fs::NamedTempFile::new("sample.txt").unwrap();
    file.write_str(":foo('test')+").unwrap();
    let path = file.path().to_str().unwrap();

    let mut cmd = command(&["-p", path]);
    cmd.assert().success().stdout("(?P<foo>test)+\n");

    let mut cmd = command(&["-p", path, "-fJS"]);
    cmd.assert().success().stdout("(?<foo>test)+\n");

    let mut cmd = command(&["-fJS", "-p", path]);
    cmd.assert().success().stdout("(?<foo>test)+\n");

    file.write_binary(b"\xC3\x28").unwrap();
    let path = file.path().to_str().unwrap();

    let mut cmd = command(&["-fJS", "-p", path]);
    cmd.assert()
        .failure()
        .stdout("")
        .stderr(format!("{ERROR}stream did not contain valid UTF-8\n"));
}

#[test]
fn no_newline() {
    let mut cmd = command(&[":foo('test')+", "--no-new-line"]);
    cmd.assert().success().stdout("(?P<foo>test)+").stderr("");

    let mut cmd = command(&[":foo('test')+", "-n"]);
    cmd.assert().success().stdout("(?P<foo>test)+").stderr("");

    let mut cmd = command(&["-n", ":foo('test')+"]);
    cmd.assert().success().stdout("(?P<foo>test)+").stderr("");

    let mut cmd = command(&["-n", ":foo('test')+", "-n"]);
    cmd.assert().failure().stderr(format!(
        r#"{ERROR}The argument '--no-new-line' was provided more than once, but cannot be
  │ used multiple times
{USAGE}"#
    ));
}

#[test]
fn lots_of_warnings() {
    let mut cmd = command(&["[.][.][.][.][.][.][.][.][.][.][.][.]"]);
    cmd.assert().success().stdout("............\n").stderr(
        r#"warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·  ┬
   ·  ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·     ┬
   ·     ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·        ┬
   ·        ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·           ┬
   ·           ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·              ┬
   ·              ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·                 ┬
   ·                 ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·                    ┬
   ·                    ╰── warning originated here
   ╰────
warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·                       ┬
   ·                       ╰── warning originated here
   ╰────
note: some warnings were omitted
warning: pomsky generated 12 warnings
"#,
    );
}

#[test]
fn disable_warnings() {
    let mut cmd = command(&["[.]", "-W0"]);
    cmd.assert().success().stdout(".\n").stderr("");

    let mut cmd = command(&["[.]", "-Wdeprecated=0"]);
    cmd.assert().success().stdout(".\n").stderr("");

    let mut cmd = command(&["[.]", "-Wcompat=0"]);
    cmd.assert().success().stdout(".\n").stderr(
        r#"warning P0105(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.]
   ·  ┬
   ·  ╰── warning originated here
   ╰────
"#,
    );
}

#[test]
fn wrong_order() {
    let mut cmd = command(&["-pf", "file.txt", "rust"]);
    cmd.assert().failure().stderr(format!("{ERROR}unexpected argument \"rust\"\n{USAGE}"));

    let mut cmd = command(&["-p", "-W0", "file.txt"]);
    cmd.assert()
        .failure()
        .stderr(format!("{ERROR}You can only provide an input or a path, but not both\n{USAGE}"));
}

#[test]
fn specify_features() {
    let mut cmd = command(&[
        ":(.)",
        "--allowed-features",
        "variables,boundaries,dot,atomic-groups,lazy-mode,named-groups",
    ]);
    cmd.assert().failure().stderr(
        r#"error P0302(syntax): 
  × Numbered capturing groups aren't supported
   ╭────
 1 │ :(.)
   · ──┬─
   ·   ╰── error occurred here
   ╰────
"#,
    );
}

#[test]
fn json_output() {
    let mut cmd = command(&["..[word]", "--json"]);
    cmd.assert()
        .success()
        .stdout(Output::new(CompilationResult {
            version: Version::V1,
            success: true,
            output: Some("..\\w".into()),
            diagnostics: vec![],
            timings: Timings { all: 0 },
        }))
        .stderr("");
}

#[test]
fn json_output_warnings() {
    let mut cmd = command(&["[.][.]", "--json"]);
    cmd.assert()
        .success()
        .stdout(Output::new(CompilationResult {
            version: Version::V1,
            success: true,
            output: Some("..".into()),
            diagnostics: vec![
                Diagnostic {
                    severity: Severity::Warning,
                    kind: Kind::Deprecated,
                    code: Some(DiagnosticCode::DeprecatedSyntax),
                    spans: vec![Span { start: 1, end: 2, label: None }],
                    description: "This syntax is deprecated. Use `.` without the brackets.".into(),
                    help: vec![],
                    fixes: vec![],
                    visual: String::new(),
                },
                Diagnostic {
                    severity: Severity::Warning,
                    kind: Kind::Deprecated,
                    code: Some(DiagnosticCode::DeprecatedSyntax),
                    spans: vec![Span { start: 4, end: 5, label: None }],
                    description: "This syntax is deprecated. Use `.` without the brackets.".into(),
                    help: vec![],
                    fixes: vec![],
                    visual: String::new(),
                },
            ],
            timings: Timings { all: 0 },
        }))
        .stderr("");
}

#[test]
fn json_output_errors() {
    let mut cmd = command(&["[cp][^test]", "--json"]);
    cmd.assert()
        .failure()
        .stdout(
            Output::new(CompilationResult {
                version: Version::V1,
                success: false,
                output: None,
                diagnostics: vec![Diagnostic {
                    severity: Severity::Error,
                    kind: Kind::Deprecated,
                    code: Some(DiagnosticCode::DeprecatedSyntax),
                    spans: vec![Span { start: 1, end: 3, label: None }],
                    description: "`[cp]` is deprecated".into(),
                    help: vec!["Use `C` without brackets instead".into()],
                    fixes: vec![],
                    visual: String::from(
                        "error P0105(deprecated): 
  × `[cp]` is deprecated
   ╭────
 1 │ [cp][^test]
   ·  ─┬
   ·   ╰── error occurred here
   ╰────
  help: Use `C` without brackets instead
",
                    ),
                }],
                timings: Timings { all: 0 },
            })
            .ignore_visual(false),
        )
        .stderr("");
}

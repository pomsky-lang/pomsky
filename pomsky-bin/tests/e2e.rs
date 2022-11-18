// these tests don't work on Windows for some reason
#![cfg(not(target_os = "windows"))]

use assert_cmd::prelude::*;
use assert_fs::prelude::FileWriteStr;
use std::process::Command;

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

    let mut cmd = command(&["--debug"]);
    cmd.assert().success().stdout("\n").stderr(
        r#"======================== debug ========================
""

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
        r#"{ERROR}The argument 'no-new-line' was provided more than once, but cannot be used
  │ multiple times
{USAGE}"#
    ));
}

#[test]
fn lots_of_warnings() {
    let mut cmd = command(&["[.][.][.][.][.][.][.][.][.][.][.][.]"]);
    cmd.assert().success().stdout("............\n").stderr(
        r#"warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·  ┬
   ·  ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·     ┬
   ·     ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·        ┬
   ·        ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·           ┬
   ·           ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·              ┬
   ·              ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·                 ┬
   ·                 ╰── warning originated here
   ╰────
warning(deprecated): 
  ⚠ This syntax is deprecated. Use `.` without the brackets.
   ╭────
 1 │ [.][.][.][.][.][.][.][.][.][.][.][.]
   ·                    ┬
   ·                    ╰── warning originated here
   ╰────
warning(deprecated): 
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
        r#"warning(deprecated): 
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

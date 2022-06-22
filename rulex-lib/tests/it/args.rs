use std::process::exit;

use crate::color::Color::*;

pub(crate) struct Args {
    pub include_ignored: bool,
    pub filter: String,
    pub bless: bool,

    pub fuzz_ranges: bool,
    pub fuzz_start: usize,
    pub fuzz_step: usize,
    pub thoroughness: usize,
}

impl Args {
    pub(crate) fn parse() -> Self {
        let mut include_ignored = false;
        let mut filter = String::new();
        let mut bless = false;
        let mut fuzz_ranges = false;
        let mut fuzz_start = 0;
        let mut fuzz_step = 1;
        let mut thoroughness = 40;

        for arg in std::env::args().skip(1) {
            match arg.as_str() {
                "--" => {}
                "-i" | "--ignored" | "--include-ignored" => include_ignored = true,
                "--fuzz-ranges" => fuzz_ranges = true,
                "--bless" => bless = true,
                "help" | "--help" | "-h" => {
                    eprintln!(
                        "USAGE:\n    \
                            cargo test --test it -- [OPTIONS]\n\
                        \n\
                        OPTIONS:\n    \
                            -i,--ignored            Include ignored test cases\n    \
                            --bless                 Bless failed test cases\n    \
                            --fuzz-ranges           Fuzz the `range '...'-'...' syntax`\n    \
                            --thoroughness=<NUMBER> Specify how thorough each range is fuzzed [default: 40]\n    \
                            --fuzz-start=<NUMBER>   Specify the bound where to start fuzzing [default: 0]\n    \
                            --fuzz-step=<NUMBER>    Only fuzz every n-th number (use prime number to make samples more arbitrary)\n    \
                            -h,--help               Show usage information"
                    );
                    exit(0);
                }
                s if s.starts_with("--thoroughness=") => {
                    let s = s.strip_prefix("--thoroughness=").unwrap();
                    thoroughness = s.parse().unwrap();
                }
                s if s.starts_with("--fuzz-start=") => {
                    let s = s.strip_prefix("--fuzz-start=").unwrap();
                    fuzz_start = s.parse().unwrap();
                }
                s if s.starts_with("--fuzz-step=") => {
                    let s = s.strip_prefix("--fuzz-step=").unwrap();
                    fuzz_step = s.parse().unwrap();
                }
                s if !s.starts_with('-') => filter = arg,
                option => eprintln!(
                    "{}: unrecognized option {option:?}\ntry `--help` help",
                    Yellow("Warning")
                ),
            }
        }
        Args { include_ignored, filter, bless, fuzz_ranges, fuzz_start, fuzz_step, thoroughness }
    }
}

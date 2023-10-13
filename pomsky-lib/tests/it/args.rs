use std::process::exit;

pub(crate) struct Args {
    pub include_ignored: bool,
    pub filter: String,
    pub bless: bool,
    pub stats: bool,

    pub fuzz_ranges: bool,
    pub fuzz_start: usize,
    pub fuzz_step: usize,
    pub thoroughness: usize,
}

impl Args {
    pub(crate) fn parse() -> Self {
        use lexopt::prelude::*;

        let mut include_ignored = false;
        let mut filter = String::new();
        let mut bless = false;
        let mut stats = false;

        let mut fuzz_ranges = false;
        let mut fuzz_start = 0;
        let mut fuzz_step = 1;
        let mut thoroughness = 40;

        let mut parser = lexopt::Parser::from_env();
        while let Some(arg) = parser.next().unwrap() {
            match arg {
                Short('i') | Long("ignored") | Long("include-ignored") => include_ignored = true,
                Long("fuzz-ranges") => fuzz_ranges = true,
                Long("bless") => bless = true,
                Long("stats") => stats = true,
                Short('h') | Long("help") => {
                    eprintln!(
                        "USAGE:\n    \
                            cargo test --test it -- [OPTIONS]\n\
                        \n\
                        OPTIONS:\n    \
                            -i,--ignored            Include ignored test cases\n    \
                            --bless                 Bless failed test cases\n    \
                            --stats                 Show some statistics\n    \
                            --fuzz-ranges           Fuzz the `range '...'-'...' syntax`\n    \
                            --thoroughness=<NUMBER> Specify how thorough each range is fuzzed [default: 40]\n    \
                            --fuzz-start=<NUMBER>   Specify the bound where to start fuzzing [default: 0]\n    \
                            --fuzz-step=<NUMBER>    Only fuzz every n-th number (use prime number to make samples more arbitrary)\n    \
                            -h,--help               Show usage information"
                    );
                    exit(0);
                }
                Long("thoroughness") => thoroughness = parser.value().unwrap().parse().unwrap(),
                Long("fuzz-start") => fuzz_start = parser.value().unwrap().parse().unwrap(),
                Long("fuzz-step") => fuzz_step = parser.value().unwrap().parse().unwrap(),
                Value(arg) => filter = arg.to_string_lossy().to_string(),
                _ => {
                    eprintln!("error: {}", arg.unexpected());
                    std::process::exit(1);
                }
            }
        }
        Args {
            include_ignored,
            filter,
            bless,
            stats,
            fuzz_ranges,
            fuzz_start,
            fuzz_step,
            thoroughness,
        }
    }
}

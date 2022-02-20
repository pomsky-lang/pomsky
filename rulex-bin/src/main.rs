use std::{io::Read, path::PathBuf};

use atty::Stream;
use clap::{ArgEnum, Parser};
use rulex::{
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

/// Compile a rulex expression to a regex
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rulex expression to compile
    input: Option<String>,
    /// File containing the rulex expression to compile
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    path: Option<PathBuf>,

    /// Show debug information
    #[clap(short, long)]
    debug: bool,

    /// Regex flavor
    #[clap(long, short, arg_enum, ignore_case(true))]
    flavor: Option<Flavor>,
}

/// Regex flavor
#[derive(Clone, Debug, ArgEnum)]
enum Flavor {
    Pcre,
    Python,
    Java,
    JavaScript,
    DotNet,
    Ruby,
    Rust,
}

impl From<Flavor> for RegexFlavor {
    fn from(f: Flavor) -> Self {
        match f {
            Flavor::Pcre => RegexFlavor::Pcre,
            Flavor::Python => RegexFlavor::Python,
            Flavor::Java => RegexFlavor::Java,
            Flavor::JavaScript => RegexFlavor::JavaScript,
            Flavor::DotNet => RegexFlavor::DotNet,
            Flavor::Ruby => RegexFlavor::Ruby,
            Flavor::Rust => RegexFlavor::Rust,
        }
    }
}

pub fn main() {
    let args = Args::parse();

    match (args.input, args.path) {
        (Some(input), None) => compile(&input, args.debug, args.flavor),
        (None, Some(path)) => match std::fs::read_to_string(path) {
            Ok(input) => compile(&input, args.debug, args.flavor),
            Err(e) => eprintln!("error reading file: {e}"),
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input, args.debug, args.flavor),
                Err(e) => eprintln!("error parsing stdin: {e}"),
            }
        }
        (Some(_), Some(_)) => eprintln!("error: Can't provide an input and a path"),
        (None, None) => eprintln!("error: No input provided"),
    }
}

fn compile(input: &str, debug: bool, flavor: Option<Flavor>) {
    let parsed = match Rulex::parse(input, Default::default()) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("error: {e:?}");
            return;
        }
    };
    if debug {
        println!("{parsed:#?}");
    }
    match parsed.compile(CompileOptions {
        flavor: flavor.unwrap_or(Flavor::Pcre).into(),
        ..Default::default()
    }) {
        Ok(compiled) => println!("{compiled}"),
        Err(e) => eprintln!("{e:?}"),
    }
}

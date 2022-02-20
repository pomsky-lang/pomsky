use std::{io::Read, path::PathBuf};

use atty::Stream;
use clap::Parser;
use rulex::Rulex;

/// Compile a rulex expression to a regex
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rulex expression to compile
    input: Option<String>,
    /// File containing the rulex expression to compile
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    path: Option<PathBuf>,
}

pub fn main() {
    let cli = Args::parse();

    match (cli.input, cli.path) {
        (Some(input), None) => compile(&input),
        (None, Some(path)) => match std::fs::read_to_string(path) {
            Ok(input) => compile(&input),
            Err(e) => eprintln!("error reading file: {e}"),
        },
        (None, None) if atty::isnt(Stream::Stdin) => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();

            match String::from_utf8(buf) {
                Ok(input) => compile(&input),
                Err(e) => eprintln!("error parsing stdin: {e}"),
            }
        }
        (Some(_), Some(_)) => eprintln!("error: Can't provide an input and a path"),
        (None, None) => eprintln!("error: No input provided"),
    }
}

fn compile(input: &str) {
    let parsed = match Rulex::parse(input, Default::default()) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("error: {e:?}");
            return;
        }
    };
    println!("{parsed:#?}");
}

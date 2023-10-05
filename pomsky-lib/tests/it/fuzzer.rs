use std::{fmt, time::Instant};

use crate::color::Color::*;
use pomsky::Expr;
use regex::Regex;

#[derive(Debug)]
pub(crate) enum FuzzError {
    FalsePositive { range: (usize, usize), regex: String, input: String },
    FalseNegative { range: (usize, usize), regex: String, input: String },
    InvalidRegex { range: (usize, usize), regex: String, error: regex::Error },
}

impl fmt::Display for FuzzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuzzError::FalsePositive { range: (start, end), regex, input } => {
                write!(f, "false positive\n      range: {start}-{end}\ntest string: {input}\n      regex: {regex}")
            }
            FuzzError::FalseNegative { range: (start, end), regex, input } => {
                write!(f, "false negative\n      range: {start}-{end}\ntest string: {input}\n      regex: {regex}")
            }
            FuzzError::InvalidRegex { range: (start, end), regex, error } => {
                write!(f, "invalid regex\n        range: {start}-{end}\nerror message: {error}\n        regex: {regex}")
            }
        }
    }
}

pub(crate) fn fuzz_ranges(
    errors: &mut Vec<FuzzError>,
    thoroughness: usize,
    start: usize,
    step: usize,
) {
    let mut strings = vec![];
    let mut max_lo = start;
    let mut max_hi = start;
    let mut lo = start;
    let mut hi = start;

    let mut start = Instant::now();
    loop {
        let (Some(regex), _warnings) =
            Expr::parse_and_compile(&format!("range '{lo}'-'{hi}'"), Default::default())
        else {
            panic!("Compiling range failed");
        };
        let regex = match Regex::new(&format!("^({regex})$")) {
            Ok(r) => r,
            Err(error) => {
                let res = FuzzError::InvalidRegex { range: (lo, hi), regex, error };
                eprintln!("{}: {res}\n", Red("Error"));
                errors.push(res);
                continue;
            }
        };

        for i in ((lo % step)..100 + lo.max(hi) * thoroughness).step_by(step) {
            while i >= strings.len() {
                strings.push(strings.len().to_string());
            }

            match (regex.is_match(&strings[i]), i >= lo && i <= hi) {
                (true, true) | (false, false) => {} // ok
                (true, false) => {
                    let res = FuzzError::FalsePositive {
                        range: (lo, hi),
                        regex: regex.as_str().to_string(),
                        input: strings[i].clone(),
                    };
                    eprintln!("{}: {res}\n", Red("Error"));
                    errors.push(res);
                }
                (false, true) => {
                    let res = FuzzError::FalseNegative {
                        range: (lo, hi),
                        regex: regex.as_str().to_string(),
                        input: strings[i].clone(),
                    };
                    eprintln!("{}: {res}\n", Red("Error"));
                    errors.push(res);
                }
            }
        }

        if lo < max_lo {
            lo += 1;

            if start.elapsed().as_secs() >= 10 {
                eprintln!("     - at {lo}-{hi}");
                start = Instant::now();
            }
        } else if hi < max_hi {
            hi += 1;

            if start.elapsed().as_secs() >= 10 {
                eprintln!("     - at {lo}-{hi}");
                start = Instant::now();
            }
        } else {
            if start.elapsed().as_secs() >= 3 {
                eprintln!("completed {max_lo}-{max_hi}");
                start = Instant::now();
            }

            if max_lo < max_hi {
                max_lo += 1;
                lo += max_lo;
                hi = lo;
            } else {
                max_hi += 1;
                hi = max_hi;
                lo = 0;
            }
        }
    }
}

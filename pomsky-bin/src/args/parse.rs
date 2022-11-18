use atty::Stream;

use super::{Args, DiagnosticSet, Input, ParseArgsError};

#[derive(PartialEq)]
pub(super) enum ArgsInner {
    Args(Args),
    HelpShort,
    HelpLong,
    Version,
}

pub(super) fn parse_args_inner(mut parser: lexopt::Parser) -> Result<ArgsInner, ParseArgsError> {
    use lexopt::prelude::*;

    let mut arg_count = 0;
    let mut input_value = None;
    let mut path = None;
    let mut debug = false;
    let mut flavor = None;
    let mut no_new_line = false;
    let mut allowed_features = None;
    let mut warnings = DiagnosticSet::All;

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
                flavor = Some(super::flavors::parse_flavor(parser.value()?)?);
            }
            Short('n') | Long("no-new-line") => {
                if no_new_line {
                    return Err(ParseArgsError::UnexpectedTwice("no-new-line"));
                }
                no_new_line = true;
            }
            Short('W') | Long("warnings") => {
                warnings = DiagnosticSet::parse(parser.value()?, warnings)?;
            }
            Long("allowed-features") => {
                if allowed_features.is_some() {
                    return Err(ParseArgsError::UnexpectedTwice("--allowed-features"));
                }
                allowed_features = Some(super::features::parse_features(parser.value()?)?);
            }
            Value(val) if input_value.is_none() => {
                input_value = Some(val.into_string().map_err(lexopt::Error::from)?);
            }
            Short('h') => {
                return Ok(ArgsInner::HelpShort);
            }
            Long("help") => {
                return Ok(ArgsInner::HelpLong);
            }
            Short('V') | Long("version") => {
                return Ok(ArgsInner::Version);
            }
            _ => Err(arg.unexpected())?,
        }
    }

    if arg_count == 0 && atty::is(Stream::Stdin) && atty::is(Stream::Stdout) {
        return Ok(ArgsInner::HelpShort);
    }

    let input = match (input_value, path) {
        (Some(input), None) => Input::Value(input),
        (None, Some(path)) => Input::File(path),
        (Some(_), Some(_)) => return Err(ParseArgsError::InputAndPath),
        (None, None) => Input::read_stdin()?,
    };

    Ok(ArgsInner::Args(Args {
        input,
        flavor,
        debug,
        no_new_line,
        allowed_features: allowed_features.unwrap_or_default(),
        warnings,
    }))
}

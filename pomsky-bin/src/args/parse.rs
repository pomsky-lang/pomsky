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
    let mut json = false;

    while let Some(arg) = parser.next()? {
        arg_count += 1;

        match arg {
            Short('p') | Long("path") => path.set_arg(parser.value()?.parse()?, "--path")?,
            Short('d') | Long("debug") => debug.set_arg(true, "--debug")?,
            Short('f') | Long("flavor") => {
                flavor.set_arg(super::flavors::parse_flavor(parser.value()?)?, "--flavor")?;
            }
            Short('n') | Long("no-new-line") => no_new_line.set_arg(true, "--no-new-line")?,
            Short('W') | Long("warnings") => {
                warnings = DiagnosticSet::parse(parser.value()?, warnings)?;
            }
            Long("allowed-features") => allowed_features
                .set_arg(super::features::parse_features(parser.value()?)?, "--allowed-features")?,
            Long("json") => json.set_arg(true, "--json")?,
            Value(val) if input_value.is_none() => {
                input_value = Some(val.into_string().map_err(lexopt::Error::from)?);
            }
            Short('h') => return Ok(ArgsInner::HelpShort),
            Long("help") => return Ok(ArgsInner::HelpLong),
            Short('V') | Long("version") => return Ok(ArgsInner::Version),
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
        json,
        no_new_line,
        allowed_features: allowed_features.unwrap_or_default(),
        warnings,
    }))
}

trait SetArg {
    type Set;

    fn set_arg(&mut self, value: Self::Set, name: &'static str) -> Result<(), ParseArgsError>;
}

impl SetArg for bool {
    type Set = bool;

    fn set_arg(&mut self, value: bool, name: &'static str) -> Result<(), ParseArgsError> {
        if *self == value {
            return Err(ParseArgsError::UnexpectedTwice(name));
        }
        *self = value;
        Ok(())
    }
}

impl<T> SetArg for Option<T> {
    type Set = T;

    fn set_arg(&mut self, value: T, name: &'static str) -> Result<(), ParseArgsError> {
        if self.is_some() {
            return Err(ParseArgsError::UnexpectedTwice(name));
        }
        *self = Some(value);
        Ok(())
    }
}

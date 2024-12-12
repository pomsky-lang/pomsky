use std::io::{stdin, stdout, IsTerminal};
use std::path::PathBuf;

use pomsky::{features::PomskyFeatures, options::RegexFlavor};

use crate::args::RegexEngine;
use crate::format::Logger;

use super::{
    CompileOptions, DiagnosticSet, GlobalOptions, Input, ParseArgsError, Subcommand, TestOptions,
};

#[derive(PartialEq)]
pub(super) enum Parsed {
    Options(Subcommand, GlobalOptions),
    Help(Help),
    Version,
    List(ListKind),
}

pub fn parse_args_inner(
    logger: &Logger,
    mut parser: lexopt::Parser,
) -> Result<Parsed, ParseArgsError> {
    RootParser::new().parse(logger, &mut parser)
}

#[derive(PartialEq)]
pub(super) enum Help {
    Short,
    Long,
    TestShort,
    TestLong,
}

#[derive(PartialEq)]
pub(super) enum ListKind {
    Shorthands,
}

struct RootParser {
    debug: bool,
    flavor: Option<RegexFlavor>,
    allowed_features: Option<PomskyFeatures>,
    warnings: DiagnosticSet,
    json: Option<bool>,
}

// we have to do this to deduplicate code without creating borrowcheck issues, because `parser` is a lending iterator
macro_rules! parse_root_arg {
    ($logger:expr, $arg:expr, $parser:expr, $root:expr) => {
        match $arg {
            Short('d') | Long("debug") => $root.debug.set_arg(true, "--debug")?,
            Short('f') | Long("flavor") => {
                $root
                    .flavor
                    .set_arg(super::flavors::parse_flavor($parser.value()?)?, "--flavor")?;
            }
            Short('W') | Long("warnings") => $root.warnings.parse($parser.value()?)?,
            Long("allowed-features") => $root.allowed_features.set_arg(
                super::features::parse_features($logger, $parser.value()?)?,
                "--allowed-features",
            )?,
            Long("json") => $root.json.set_arg(true, "--json")?,
            Short('h') => return Ok(Parsed::Help(Help::Short)),
            Long("help") => return Ok(Parsed::Help(Help::Long)),
            Short('V') | Long("version") => return Ok(Parsed::Version),
            _ => Err($arg.unexpected())?,
        }
    };
}

macro_rules! parse_compile_options {
    ($logger:expr, $arg:expr, $parser:expr, $self:expr) => {
        match $arg {
            Short('p') | Long("path") => $self.path.set_arg($parser.value()?.parse()?, "--path")?,
            Short('t') | Long("test") => {
                $self.test.set_arg(RegexEngine::parse($parser.value()?)?, "--test")?;
            }
            Short('n') | Long("no-new-line") => $self.no_new_line.set_arg(true, "--no-new-line")?,
            Value(val) if $self.input_value.is_none() => {
                $self.input_value = Some(val.into_string().map_err(lexopt::Error::from)?);
            }
            _ => parse_root_arg!($logger, $arg, $parser, $self.root),
        }
    };
}

impl RootParser {
    fn new() -> Self {
        Self {
            debug: false,
            flavor: None,
            allowed_features: None,
            warnings: DiagnosticSet::default(),
            json: None,
        }
    }

    fn parse(self, logger: &Logger, parser: &mut lexopt::Parser) -> Result<Parsed, ParseArgsError> {
        use lexopt::prelude::*;

        let Some(arg) = parser.next()? else {
            if stdin().is_terminal() && stdout().is_terminal() {
                return Ok(Parsed::Help(Help::Short));
            } else {
                return CompileParser::new(self).finish();
            }
        };
        match arg {
            Long("list") => {
                let list_arg = parser.value()?.string()?;
                if &list_arg != "shorthands" {
                    return Err(ParseArgsError::UnknownList(list_arg));
                };
                Ok(Parsed::List(ListKind::Shorthands))
            }
            Value(val) if val == "test" => TestParser::new(self).parse(logger, parser),
            arg => {
                let mut compile_parser = CompileParser::new(self);
                parse_compile_options!(logger, arg, parser, compile_parser);
                compile_parser.parse(logger, parser)
            }
        }
    }

    fn finish(self, subcommand: Subcommand) -> Result<Parsed, ParseArgsError> {
        Ok(Parsed::Options(
            subcommand,
            GlobalOptions {
                flavor: self.flavor,
                debug: self.debug,
                json: self.json.unwrap_or_default(),
                allowed_features: self.allowed_features.unwrap_or_default(),
                warnings: self.warnings,
            },
        ))
    }
}

struct CompileParser {
    root: RootParser,
    input_value: Option<String>,
    path: Option<PathBuf>,
    no_new_line: bool,
    test: Option<RegexEngine>,
}

impl CompileParser {
    fn new(root: RootParser) -> Self {
        Self { root, input_value: None, path: None, no_new_line: false, test: None }
    }

    fn parse(
        mut self,
        logger: &Logger,
        parser: &mut lexopt::Parser,
    ) -> Result<Parsed, ParseArgsError> {
        use lexopt::prelude::*;

        while let Some(arg) = parser.next()? {
            parse_compile_options!(logger, arg, parser, self);
        }
        self.finish()
    }

    fn finish(self) -> Result<Parsed, ParseArgsError> {
        let input = match (self.input_value, self.path) {
            (Some(input), None) => Input::Value(input),
            (None, Some(path)) => Input::File(path),
            (Some(_), Some(_)) => return Err(ParseArgsError::InputAndPath),
            (None, None) => Input::read_stdin()?,
        };

        self.root.finish(Subcommand::Compile(CompileOptions {
            input,
            no_new_line: self.no_new_line,
            test: self.test,
            in_test_suite: false,
        }))
    }
}

struct TestParser {
    root: RootParser,
    path: Option<PathBuf>,
    engine: Option<RegexEngine>,
    pass_with_no_tests: Option<bool>,
}

impl TestParser {
    fn new(root: RootParser) -> Self {
        Self { root, path: None, engine: None, pass_with_no_tests: None }
    }

    fn parse(
        mut self,
        logger: &Logger,
        parser: &mut lexopt::Parser,
    ) -> Result<Parsed, ParseArgsError> {
        use lexopt::prelude::*;

        while let Some(arg) = parser.next()? {
            match arg {
                Short('p') | Long("path") => {
                    self.path.set_arg(parser.value()?.parse()?, "--path")?
                }
                Short('e') | Long("engine") => {
                    self.engine.set_arg(RegexEngine::parse(parser.value()?)?, "--engine")?
                }
                Long("pass-with-no-tests") => {
                    self.pass_with_no_tests.set_arg(true, "--pass-with-no-tests")?
                }
                Short('h') => return Ok(Parsed::Help(Help::TestShort)),
                Long("help") => return Ok(Parsed::Help(Help::TestLong)),
                _ => parse_root_arg!(logger, arg, parser, self.root),
            }
        }
        self.finish()
    }

    fn finish(self) -> Result<Parsed, ParseArgsError> {
        let path = self.path.ok_or(ParseArgsError::NoPath)?;

        self.root.finish(Subcommand::Test(TestOptions {
            path,
            engine: self.engine,
            pass_with_no_tests: self.pass_with_no_tests.unwrap_or_default(),
        }))
    }
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

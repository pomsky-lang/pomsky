use std::collections::HashMap;

use pomsky_syntax::exprs::{Capture, Group};

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Group<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self.capture {
            Some(Capture { name: Some(name) }) => {
                if within_variable {
                    return Err(CompileErrorKind::CaptureInLet.at(self.span));
                }

                if map.contains_key(name) {
                    return Err(
                        CompileErrorKind::NameUsedMultipleTimes(name.to_string()).at(self.span)
                    );
                }

                *count += 1;
                map.insert(name.to_string(), *count);
            }
            Some(Capture { name: None }) => {
                if within_variable {
                    return Err(CompileErrorKind::CaptureInLet.at(self.span));
                }

                *count += 1;
            }
            None => {}
        };
        for rule in &self.parts {
            rule.get_capturing_groups(count, map, within_variable)?;
        }
        Ok(())
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        if self.capture.is_some() {
            state.next_idx += 1;
        }

        Ok(Regex::Group(RegexGroup {
            parts: self
                .parts
                .iter()
                .map(|part| part.compile(options, state))
                .collect::<Result<_, _>>()?,
            capture: match self.capture {
                Some(Capture { name: Some(name) }) => RegexCapture::NamedCapture(name),
                Some(Capture { name: None }) => RegexCapture::Capture,
                None => RegexCapture::None,
            },
        }))
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        for rule in &self.parts {
            rule.validate(options)?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexGroup<'i> {
    parts: Vec<Regex<'i>>,
    capture: RegexCapture<'i>,
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexCapture<'i> {
    Capture,
    NamedCapture(&'i str),
    None,
    NoneWithParens,
}

impl<'i> RegexGroup<'i> {
    pub(crate) fn new(parts: Vec<Regex<'i>>, capture: RegexCapture<'i>) -> Self {
        Self { parts, capture }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match self.capture {
            RegexCapture::NamedCapture(name) => {
                // https://www.regular-expressions.info/named.html
                match flavor {
                    RegexFlavor::Python | RegexFlavor::Pcre | RegexFlavor::Rust => {
                        buf.push_str("(?P<");
                    }
                    RegexFlavor::DotNet
                    | RegexFlavor::Java
                    | RegexFlavor::Ruby
                    | RegexFlavor::JavaScript => {
                        buf.push_str("(?<");
                    }
                }
                buf.push_str(name);
                buf.push('>');
                for part in &self.parts {
                    part.codegen(buf, flavor);
                }
                buf.push(')');
            }
            RegexCapture::Capture => {
                buf.push('(');
                for part in &self.parts {
                    part.codegen(buf, flavor);
                }
                buf.push(')');
            }
            RegexCapture::None => {
                for part in &self.parts {
                    let needs_parens = part.needs_parens_in_group();
                    if needs_parens {
                        buf.push_str("(?:");
                    }
                    part.codegen(buf, flavor);
                    if needs_parens {
                        buf.push(')');
                    }
                }
            }
            RegexCapture::NoneWithParens => {
                for part in &self.parts {
                    buf.push_str("(?:");
                    part.codegen(buf, flavor);
                    buf.push(')');
                }
            }
        }
    }

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        match self.capture {
            RegexCapture::None if self.parts.len() == 1 => {
                self.parts[0].needs_parens_before_repetition()
            }
            RegexCapture::NoneWithParens => false,
            _ => true,
        }
    }
}

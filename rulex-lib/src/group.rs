use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    rule::Rule,
    span::Span,
};

/// A group, i.e. sequence of rules. A group is either capturing or non-capturing.
///
/// If it is capturing, it must be wrapped in parentheses, and can have a name.
/// If it is non-capturing, the parentheses can be omitted in same cases.
#[derive(Clone)]
pub(crate) struct Group<'i> {
    parts: Vec<Rule<'i>>,
    capture: Option<Capture<'i>>,
    pub(crate) span: Span,
}

impl<'i> Group<'i> {
    pub(crate) fn new(parts: Vec<Rule<'i>>, capture: Option<Capture<'i>>, span: Span) -> Self {
        Group { parts, capture, span }
    }

    pub(crate) fn set_capture(&mut self, capture: Capture<'i>) {
        self.capture = Some(capture);
    }
    pub(crate) fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }

    pub(crate) fn get_capturing_groups(
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
        for rulex in &self.parts {
            rulex.get_capturing_groups(count, map, within_variable)?;
        }
        Ok(())
    }

    pub(crate) fn compile<'c>(
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
                .map(|part| part.comp(options, state))
                .collect::<Result<_, _>>()?,
            capture: match self.capture {
                Some(Capture { name: Some(name) }) => RegexCapture::NamedCapture(name),
                Some(Capture { name: None }) => RegexCapture::Capture,
                None => RegexCapture::None,
            },
        }))
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.capture {
            Some(Capture { name: Some(name) }) => write!(f, "Group :{name}")?,
            Some(_) => write!(f, "Group :")?,
            None => write!(f, "Group")?,
        }
        if self.parts.is_empty() {
            write!(f, "()")
        } else {
            let mut tup = f.debug_tuple("");
            let mut tup = &mut tup;
            for part in &self.parts {
                tup = tup.field(part);
            }
            tup.finish()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct Capture<'i> {
    pub(crate) name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub(crate) fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

pub(crate) struct RegexGroup<'i> {
    parts: Vec<Regex<'i>>,
    capture: RegexCapture<'i>,
}

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

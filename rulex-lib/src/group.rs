use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    span::Span,
    Rulex,
};

/// A group, i.e. sequence of rules. A group is either capturing or non-capturing.
///
/// If it is capturing, it must be wrapped in parentheses, and can have a name.
/// If it is non-capturing, the parentheses can be omitted in same cases.
#[derive(Clone, PartialEq, Eq)]
pub struct Group<'i> {
    parts: Vec<Rulex<'i>>,
    capture: Option<Capture<'i>>,
    pub(crate) span: Span,
}

impl<'i> Group<'i> {
    pub(crate) fn new(parts: Vec<Rulex<'i>>, capture: Option<Capture<'i>>, span: Span) -> Self {
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
    ) -> Result<(), CompileError> {
        match self.capture {
            Some(Capture { name: Some(name) }) => {
                if map.contains_key(name) {
                    return Err(
                        CompileErrorKind::NameUsedMultipleTimes(name.to_string()).at(self.span)
                    );
                }

                *count += 1;
                map.insert(name.to_string(), *count);
            }
            Some(Capture { name: None }) => {
                *count += 1;
            }
            None => {}
        };
        for rulex in &self.parts {
            rulex.get_capturing_groups(count, map)?;
        }
        Ok(())
    }

    pub(crate) fn compile(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
    ) -> CompileResult<'i> {
        match self.capture {
            Some(_) => {
                state.next_idx += 1;
            }
            None => {}
        }

        Ok(Regex::Group(RegexGroup {
            parts: self
                .parts
                .iter()
                .map(|part| part.comp(options, state))
                .collect::<Result<_, _>>()?,
            capture: self.capture,
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
pub struct Capture<'i> {
    pub(crate) name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub(crate) fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

pub(crate) struct RegexGroup<'i> {
    parts: Vec<Regex<'i>>,
    capture: Option<Capture<'i>>,
}

impl<'i> RegexGroup<'i> {
    pub(crate) fn new(parts: Vec<Regex<'i>>, capture: Option<Capture<'i>>) -> Self {
        Self { parts, capture }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match self.capture {
            Some(Capture { name: Some(name) }) => {
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
            Some(Capture { name: None }) => {
                buf.push('(');
                for part in &self.parts {
                    part.codegen(buf, flavor);
                }
                buf.push(')');
            }
            None => {
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
        }
    }

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        if self.capture.is_none() && self.parts.len() == 1 {
            return self.parts[0].needs_parens_before_repetition();
        }
        true
    }
}

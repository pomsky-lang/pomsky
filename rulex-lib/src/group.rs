use crate::{
    compile::{Compile, CompileResult, CompileState, Transform, TransformState},
    error::CompileErrorKind,
    options::{CompileOptions, RegexFlavor},
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

    pub(crate) fn needs_parens_before_repetition(&self) -> bool {
        if self.capture.is_none() && self.parts.len() == 1 {
            return self.parts[0].needs_parens_before_repetition();
        }
        true
    }

    pub(crate) fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }

    pub(crate) fn count_capturing_groups(&self) -> u32 {
        self.parts.iter().map(Rulex::count_capturing_groups).sum()
    }
}

impl Compile for Group<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        match self.capture {
            Some(Capture { name: Some(name) }) => {
                if state.used_names.contains_key(name) {
                    return Err(
                        CompileErrorKind::NameUsedMultipleTimes(name.to_string()).at(self.span)
                    );
                }
                state.used_names.insert(name.to_string(), state.next_idx);
                state.unknown_references.retain(|(s, _)| s != name);
                state.next_idx += 1;

                // https://www.regular-expressions.info/named.html
                match options.flavor {
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
                    part.comp(options, state, buf)?;
                }
                buf.push(')');
                Ok(())
            }
            Some(Capture { name: None }) => {
                state.next_idx += 1;

                buf.push('(');
                for part in &self.parts {
                    part.comp(options, state, buf)?;
                }
                buf.push(')');
                Ok(())
            }
            None => {
                for part in &self.parts {
                    let needs_parens = part.needs_parens_in_group();
                    if needs_parens {
                        buf.push_str("(?:");
                    }
                    part.comp(options, state, buf)?;
                    if needs_parens {
                        buf.push(')');
                    }
                }
                Ok(())
            }
        }
    }
}

impl Transform for Group<'_> {
    fn transform(&mut self, options: CompileOptions, state: &mut TransformState) -> CompileResult {
        if self.capture.is_some() {
            state.next_idx += 1;
        }
        for rulex in &mut self.parts {
            rulex.transform(options, state)?;
        }
        Ok(())
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

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub struct Capture<'i> {
    name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub(crate) fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

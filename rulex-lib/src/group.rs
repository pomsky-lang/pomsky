use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileErrorKind, Feature},
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
    pub fn new(parts: Vec<Rulex<'i>>, capture: Option<Capture<'i>>, span: Span) -> Self {
        Group {
            parts,
            capture,
            span,
        }
    }

    pub fn two(a: Rulex<'i>, b: Rulex<'i>, span: Span) -> Self {
        Group {
            parts: vec![a, b],
            capture: None,
            span,
        }
    }

    pub fn set_capture(&mut self, capture: Option<Capture<'i>>) {
        self.capture = capture;
    }

    pub fn needs_parens_before_repetition(&self) -> bool {
        if self.capture.is_none() && self.parts.len() == 1 {
            return self.parts[0].needs_parens_before_repetition();
        }
        true
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
                state.next_idx += 1;

                // https://www.regular-expressions.info/named.html
                match options.flavor {
                    RegexFlavor::Python | RegexFlavor::Pcre => {
                        buf.push_str("(?P<");
                    }
                    RegexFlavor::DotNet | RegexFlavor::Java | RegexFlavor::Ruby => {
                        buf.push_str("(?<");
                    }
                    RegexFlavor::JavaScript | RegexFlavor::Rust => {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::NamedCaptureGroups,
                            options.flavor,
                        )
                        .at(self.span))
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
    pub fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileError, Feature},
    options::{CompileOptions, RegexFlavor},
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Group<'i> {
    parts: Vec<Rulex<'i>>,
    capture: Option<Capture<'i>>,
}

impl<'i> Group<'i> {
    pub fn new(parts: Vec<Rulex<'i>>, capture: Option<Capture<'i>>) -> Self {
        Group { parts, capture }
    }

    pub fn two(a: Rulex<'i>, b: Rulex<'i>) -> Self {
        Group {
            parts: vec![a, b],
            capture: None,
        }
    }

    pub fn set_capture(&mut self, capture: Option<Capture<'i>>) {
        self.capture = capture;
    }

    pub fn needs_parens_before_repetition(&self) -> bool {
        if self.capture.is_none() && self.parts.len() == 1 {
            return self.parts[0].needs_parens_before_repetition();
        }
        false
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
                    return Err(CompileError::NameUsedMultipleTimes(name.to_string()));
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
                        return Err(CompileError::Unsupported(
                            Feature::NamedCaptureGroups,
                            options.flavor,
                        ))
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

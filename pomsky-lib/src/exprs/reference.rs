use pomsky_syntax::exprs::{Reference, ReferenceTarget};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReferenceDirection {
    Backwards,
    Forwards,
}

impl From<ReferenceDirection> for Feature {
    fn from(direction: ReferenceDirection) -> Self {
        match direction {
            ReferenceDirection::Backwards => Feature::Backreference,
            ReferenceDirection::Forwards => Feature::ForwardReference,
        }
    }
}

impl<'i> RuleExt<'i> for Reference<'i> {
    fn compile(&self, options: CompileOptions, state: &mut CompileState) -> CompileResult<'i> {
        let (direction, number) = match self.target {
            ReferenceTarget::Named(name) => match state.used_names.get(name) {
                Some(&n) => {
                    let direction = if n >= state.next_idx {
                        ReferenceDirection::Forwards
                    } else {
                        ReferenceDirection::Backwards
                    };
                    (direction, n)
                }
                None => {
                    return Err(CompileErrorKind::UnknownReferenceName {
                        found: name.into(),
                        #[cfg(feature = "suggestions")]
                        similar: pomsky_syntax::find_suggestion(
                            name,
                            state.used_names.keys().map(String::as_str),
                        ),
                    }
                    .at(self.span));
                }
            },
            ReferenceTarget::Number(idx) => {
                if idx == 0 {
                    return Err(CompileErrorKind::UnknownReferenceNumber(0).at(self.span));
                }

                let direction = if idx > 99 {
                    return Err(CompileErrorKind::HugeReference.at(self.span));
                } else if idx > state.groups_count {
                    return Err(CompileErrorKind::UnknownReferenceNumber(idx as i32).at(self.span));
                } else if idx >= state.next_idx {
                    ReferenceDirection::Forwards
                } else {
                    ReferenceDirection::Backwards
                };
                (direction, idx)
            }
            ReferenceTarget::Relative(offset) => {
                let direction = if offset >= 0 {
                    ReferenceDirection::Forwards
                } else {
                    ReferenceDirection::Backwards
                };

                let num = match offset {
                    0 => return Err(CompileErrorKind::RelativeRefZero.at(self.span)),
                    i32::MIN..=-1 => offset + (state.next_idx as i32),
                    1..=i32::MAX => offset + (state.next_idx as i32) - 1,
                };
                if num <= 0 || (num as u32) > state.groups_count {
                    return Err(CompileErrorKind::UnknownReferenceNumber(num).at(self.span));
                }

                (direction, num as u32)
            }
        };

        match options.flavor {
            RegexFlavor::Rust => {
                Err(CompileErrorKind::Unsupported(direction.into(), options.flavor).at(self.span))
            }

            RegexFlavor::JavaScript | RegexFlavor::Python
                if direction == ReferenceDirection::Forwards =>
            {
                Err(CompileErrorKind::Unsupported(Feature::ForwardReference, options.flavor)
                    .at(self.span))
            }

            _ => Ok(Regex::Reference(match options.flavor {
                RegexFlavor::Ruby => {
                    if let Some(group_name) = state.used_names_vec[number as usize].as_ref() {
                        RegexReference::Name(group_name.clone())
                    } else if !state.has_named {
                        RegexReference::Number(number)
                    } else {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::MixedReferences,
                            options.flavor,
                        )
                        .at(self.span));
                    }
                }
                _ => RegexReference::Number(number),
            })),
        }
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) enum RegexReference {
    Number(u32),
    Name(String),
}

impl RegexReference {
    pub(crate) fn codegen(&self, buf: &mut String) {
        use std::fmt::Write;

        match self {
            &RegexReference::Number(number) => {
                debug_assert!(number <= 99);
                write!(buf, "(?:\\{number})").unwrap();
            }
            RegexReference::Name(name) => {
                write!(buf, "\\k<{name}>").unwrap();
            }
        }
    }
}

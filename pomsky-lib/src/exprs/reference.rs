use pomsky_syntax::exprs::{Reference, ReferenceTarget};

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReferenceDirection {
    Backwards,
    Forwards,
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
                            state.used_names.keys().map(|key| key.as_str()),
                        ),
                    }
                    .at(self.span));
                }
            },
            ReferenceTarget::Number(idx) => {
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
                    0 => {
                        return Err(
                            CompileErrorKind::Other("Relative references can't be 0").at(self.span)
                        )
                    }
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
            RegexFlavor::Rust => Err(CompileErrorKind::Unsupported(
                if direction == ReferenceDirection::Backwards {
                    Feature::Backreference
                } else {
                    Feature::ForwardReference
                },
                options.flavor,
            )
            .at(self.span)),
            RegexFlavor::JavaScript if direction == ReferenceDirection::Forwards => {
                Err(CompileErrorKind::Unsupported(Feature::ForwardReference, options.flavor)
                    .at(self.span))
            }
            _ => Ok(Regex::Reference(RegexReference { number })),
        }
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        options.allowed_features.require(PomskyFeatures::REFERENCES, self.span)
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexReference {
    number: u32,
}

impl RegexReference {
    pub(crate) fn codegen(&self, buf: &mut String, _: RegexFlavor) {
        use std::fmt::Write;

        debug_assert!(self.number <= 99);

        write!(buf, "\\{}", self.number).unwrap();
    }
}

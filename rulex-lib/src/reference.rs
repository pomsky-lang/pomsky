use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileErrorKind, Feature, ParseError},
    features::RulexFeatures,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    regex::Regex,
    span::Span,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Reference<'i> {
    pub(crate) target: ReferenceTarget<'i>,
    pub(crate) span: Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum ReferenceTarget<'i> {
    Named(&'i str),
    Number(u32),
    Relative(i32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReferenceDirection {
    Backwards,
    Forwards,
}

impl<'i> Reference<'i> {
    pub(crate) fn new(target: ReferenceTarget<'i>, span: Span) -> Self {
        Reference { target, span }
    }

    pub(crate) fn compile(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
    ) -> CompileResult<'i> {
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
                    return Err(
                        CompileErrorKind::UnknownReferenceName(name.to_string()).at(self.span)
                    );
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

    pub(crate) fn validate(&self, options: &ParseOptions) -> Result<(), ParseError> {
        options.allowed_features.require(RulexFeatures::REFERENCES, self.span)
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for Reference<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.target {
            ReferenceTarget::Named(n) => write!(f, "::{}", n),
            ReferenceTarget::Number(i) => write!(f, "::{}", i),
            ReferenceTarget::Relative(o) => write!(f, "::{}{}", if o < 0 { '-' } else { '+' }, o),
        }
    }
}

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

use crate::{
    compile::{Compile, CompileResult, CompileState},
    error::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    span::Span,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Reference<'i> {
    pub(crate) target: ReferenceTarget<'i>,
    pub(crate) span: Span,
}

impl<'i> Reference<'i> {
    pub(crate) fn new(target: ReferenceTarget<'i>, span: Span) -> Self {
        Reference { target, span }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl Compile for Reference<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        use std::fmt::Write;

        //TODO: Warn in JS mode when referencing an optional group

        let direction = match self.target {
            ReferenceTarget::Named(name) => {
                match options.flavor {
                    RegexFlavor::Pcre
                    | RegexFlavor::JavaScript
                    | RegexFlavor::Java
                    | RegexFlavor::DotNet
                    | RegexFlavor::Ruby => {
                        buf.push_str("\\k<");
                        buf.push_str(name);
                        buf.push('>');
                    }
                    RegexFlavor::Python => {
                        buf.push_str("(?P=");
                        buf.push_str(name);
                        buf.push(')');
                    }

                    // return error below
                    RegexFlavor::Rust => {}
                }

                if state.used_names.contains_key(name) {
                    ReferenceDirection::Backwards
                } else {
                    state.unknown_references.push((name.to_string(), self.span));
                    ReferenceDirection::Forwards
                }
            }
            ReferenceTarget::Number(idx) => {
                if idx > 99 {
                    return Err(CompileErrorKind::HugeReference.at(self.span));
                }

                match options.flavor {
                    RegexFlavor::Pcre
                    | RegexFlavor::JavaScript
                    | RegexFlavor::Java
                    | RegexFlavor::DotNet
                    | RegexFlavor::Ruby
                    | RegexFlavor::Python => {
                        write!(buf, "\\{idx}").unwrap();
                    }

                    // return error below
                    RegexFlavor::Rust => {}
                }

                if idx >= state.next_idx {
                    state.unknown_groups.push((idx, self.span));
                    ReferenceDirection::Forwards
                } else {
                    ReferenceDirection::Backwards
                }
            }
            ReferenceTarget::Relative(offset) => {
                //TODO convert relative to absolute references

                if offset >= 0 {
                    return Err(CompileErrorKind::Unsupported(
                        Feature::NonNegativeRelativeReference,
                        options.flavor,
                    )
                    .at(self.span));
                }

                match options.flavor {
                    RegexFlavor::Ruby => {
                        write!(buf, "\\k<{offset}>").unwrap();
                    }
                    RegexFlavor::Pcre => {
                        write!(buf, "\\g{{{offset}}}").unwrap();
                    }

                    RegexFlavor::DotNet
                    | RegexFlavor::Java
                    | RegexFlavor::JavaScript
                    | RegexFlavor::Python => {
                        return Err(CompileErrorKind::Unsupported(
                            Feature::RelativeReference,
                            options.flavor,
                        )
                        .at(self.span));
                    }

                    // return error below
                    RegexFlavor::Rust => {}
                }

                if offset >= 0 {
                    ReferenceDirection::Forwards
                } else {
                    ReferenceDirection::Backwards
                }
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
                //TODO: Return "unknown group name" if this name isn't found
                Err(
                    CompileErrorKind::Unsupported(Feature::ForwardReference, options.flavor)
                        .at(self.span),
                )
            }
            _ => Ok(()),
        }
    }
}

use pomsky_syntax::exprs::{Capture, Group, GroupKind};

use crate::{
    compile::{CompileResult, CompileState},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl RuleExt for Group {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        if let GroupKind::Capturing(_) = self.kind {
            state.next_idx += 1;
        }

        Ok(Regex::Group(RegexGroup {
            parts: self
                .parts
                .iter()
                .map(|part| part.compile(options, state))
                .collect::<Result<_, _>>()?,
            kind: match &self.kind {
                GroupKind::Capturing(Capture { name: Some(name) }) => {
                    RegexGroupKind::NamedCapture(name.clone())
                }
                GroupKind::Capturing(Capture { name: None }) => RegexGroupKind::Capture,
                GroupKind::Atomic => RegexGroupKind::Atomic,
                GroupKind::Normal | GroupKind::Implicit => RegexGroupKind::Normal,
            },
        }))
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexGroup {
    pub(crate) parts: Vec<Regex>,
    pub(crate) kind: RegexGroupKind,
}

#[cfg_attr(feature = "dbg", derive(Debug))]
#[derive(PartialEq, Eq)]
pub(crate) enum RegexGroupKind {
    Capture,
    NamedCapture(String),
    Atomic,
    Normal,
}

impl RegexGroup {
    pub(crate) fn new(parts: Vec<Regex>, capture: RegexGroupKind) -> Self {
        Self { parts, kind: capture }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match &self.kind {
            RegexGroupKind::NamedCapture(name) => {
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
            RegexGroupKind::Capture => {
                buf.push('(');
                for part in &self.parts {
                    part.codegen(buf, flavor);
                }
                buf.push(')');
            }
            RegexGroupKind::Atomic => {
                buf.push_str("(?>");
                for part in &self.parts {
                    part.codegen(buf, flavor);
                }
                buf.push(')');
            }
            RegexGroupKind::Normal => {
                let len = self.parts.len();

                for part in &self.parts {
                    let needs_parens = len > 1 && part.needs_parens_in_sequence()
                        || len == 1 && matches!(part, Regex::Unescaped(_));
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

    pub(crate) fn needs_parens_before_repetition(&self, flavor: RegexFlavor) -> bool {
        match self.kind {
            RegexGroupKind::Normal if self.parts.len() == 1 => {
                self.parts[0].needs_parens_before_repetition(flavor)
            }
            RegexGroupKind::Normal => true,
            _ => false,
        }
    }
}

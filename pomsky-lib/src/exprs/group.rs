use std::collections::HashMap;

use pomsky_syntax::exprs::{Capture, Group, GroupKind};

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileError, CompileErrorKind, Feature},
    features::PomskyFeatures,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
};

use super::RuleExt;

impl<'i> RuleExt<'i> for Group<'i> {
    fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        match self.kind {
            GroupKind::Capturing(Capture { name: Some(name) }) => {
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
            GroupKind::Capturing(Capture { name: None }) => {
                if within_variable {
                    return Err(CompileErrorKind::CaptureInLet.at(self.span));
                }

                *count += 1;
            }
            _ => {}
        };
        for rule in &self.parts {
            rule.get_capturing_groups(count, map, within_variable)?;
        }
        Ok(())
    }

    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        if let GroupKind::Capturing(_) = self.kind {
            state.next_idx += 1;
        }

        Ok(Regex::Group(RegexGroup {
            parts: self
                .parts
                .iter()
                .map(|part| part.compile(options, state))
                .collect::<Result<_, _>>()?,
            kind: match self.kind {
                GroupKind::Capturing(Capture { name: Some(name) }) => {
                    RegexGroupKind::NamedCapture(name)
                }
                GroupKind::Capturing(Capture { name: None }) => RegexGroupKind::Capture,
                GroupKind::Atomic => RegexGroupKind::Atomic,
                GroupKind::Normal | GroupKind::Implicit => RegexGroupKind::Normal,
            },
        }))
    }

    fn validate(&self, options: &CompileOptions) -> Result<(), CompileError> {
        if let GroupKind::Atomic = self.kind {
            options.allowed_features.require(PomskyFeatures::ATOMIC_GROUPS, self.span)?;

            // Python supports atomic groups since 3.11, but it's fairly new, so we forbid
            // it for now
            if let RegexFlavor::JavaScript | RegexFlavor::Python | RegexFlavor::Rust =
                options.flavor
            {
                return Err(CompileErrorKind::Unsupported(Feature::AtomicGroups, options.flavor)
                    .at(self.span));
            }
        } else if let GroupKind::Capturing(c) = self.kind {
            let feature = match &c.name {
                Some(_) => PomskyFeatures::NAMED_GROUPS,
                None => PomskyFeatures::NUMBERED_GROUPS,
            };

            options.allowed_features.require(feature, self.span)?;
        }

        for rule in &self.parts {
            rule.validate(options)?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "dbg", derive(Debug))]
pub(crate) struct RegexGroup<'i> {
    pub(crate) parts: Vec<Regex<'i>>,
    pub(crate) kind: RegexGroupKind<'i>,
}

#[cfg_attr(feature = "dbg", derive(Debug))]
#[derive(PartialEq, Eq)]
pub(crate) enum RegexGroupKind<'i> {
    Capture,
    NamedCapture(&'i str),
    Atomic,
    Normal,
}

impl<'i> RegexGroup<'i> {
    pub(crate) fn new(parts: Vec<Regex<'i>>, capture: RegexGroupKind<'i>) -> Self {
        Self { parts, kind: capture }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        match self.kind {
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

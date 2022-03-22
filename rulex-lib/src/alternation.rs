//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::CompileError,
    literal::Literal,
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    span::Span,
    Rulex,
};

/// An [alternation](https://www.regular-expressions.info/alternation.html). This is a list of
/// alternatives. Each alternative is a [`Rulex`].
///
/// If an alternative consists of multiple expressions (e.g. `'a' | 'b' 'c'`), that alternative is
/// a [`Rulex::Group`]. Note that a group's parentheses are removed when compiling to a regex if
/// they aren't required. In other words, `'a' | ('b' 'c')` compiles to `a|bc`.
#[derive(Clone, PartialEq, Eq)]
pub struct Alternation<'i> {
    rules: Vec<Rulex<'i>>,
    pub(crate) span: Span,
}

impl<'i> Alternation<'i> {
    pub(crate) fn new_rulex(rules: Vec<Rulex<'i>>) -> Rulex<'i> {
        rules
            .into_iter()
            .reduce(|a, b| match (a, b) {
                (Rulex::Alternation(mut a), Rulex::Alternation(b)) => {
                    a.rules.extend(b.rules);
                    Rulex::Alternation(a)
                }
                (Rulex::Alternation(mut a), b) => {
                    a.rules.push(b);
                    Rulex::Alternation(a)
                }
                (a, b) => {
                    let span = a.span().join(b.span());
                    Rulex::Alternation(Alternation { rules: vec![a, b], span })
                }
            })
            .unwrap_or_else(|| Rulex::Literal(Literal::new("", Span::default())))
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
    ) -> Result<(), CompileError> {
        for rule in &self.rules {
            rule.get_capturing_groups(count, map)?;
        }
        Ok(())
    }

    pub(crate) fn compile(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
    ) -> CompileResult<'i> {
        Ok(Regex::Alternation(RegexAlternation {
            parts: self
                .rules
                .iter()
                .map(|rule| rule.comp(options, state))
                .collect::<Result<_, _>>()?,
        }))
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Alternation<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("Alternation");
        let mut d = &mut d;
        for rule in &self.rules {
            d = d.field(rule);
        }
        d.finish()
    }
}

pub(crate) struct RegexAlternation<'i> {
    parts: Vec<Regex<'i>>,
}

impl<'i> RegexAlternation<'i> {
    pub(crate) fn new(parts: Vec<Regex<'i>>) -> Self {
        Self { parts }
    }

    pub(crate) fn codegen(&self, buf: &mut String, flavor: RegexFlavor) {
        for rule in &self.parts {
            rule.codegen(buf, flavor);
            buf.push('|');
        }
        let _ = buf.pop();
    }
}

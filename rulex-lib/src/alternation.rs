//! Implements [alternation](https://www.regular-expressions.info/alternation.html):
//! `('alt1' | 'alt2' | 'alt3')`.

use std::{borrow::Cow, collections::HashMap};

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, ParseError},
    literal::Literal,
    options::{CompileOptions, ParseOptions, RegexFlavor},
    regex::Regex,
    rule::Rule,
    span::Span,
};

/// An [alternation](https://www.regular-expressions.info/alternation.html).
/// This is a list of alternatives. Each alternative is a [`Rulex`].
///
/// If an alternative consists of multiple expressions (e.g. `'a' | 'b' 'c'`),
/// that alternative is a [`Rulex::Group`]. Note that a group's parentheses are
/// removed when compiling to a regex if they aren't required. In other words,
/// `'a' | ('b' 'c')` compiles to `a|bc`.
#[derive(Clone)]
pub(crate) struct Alternation<'i> {
    rules: Vec<Rule<'i>>,
    pub(crate) span: Span,
}

impl<'i> Alternation<'i> {
    pub(crate) fn new_rulex(rules: Vec<Rule<'i>>) -> Rule<'i> {
        rules
            .into_iter()
            .reduce(|a, b| match (a, b) {
                (Rule::Alternation(mut a), Rule::Alternation(b)) => {
                    a.span = a.span.join(b.span);
                    a.rules.extend(b.rules);
                    Rule::Alternation(a)
                }
                (Rule::Alternation(mut a), b) => {
                    a.span = a.span.join(b.span());
                    a.rules.push(b);
                    Rule::Alternation(a)
                }
                (a, b) => {
                    let span = a.span().join(b.span());
                    Rule::Alternation(Alternation { rules: vec![a, b], span })
                }
            })
            .unwrap_or_else(|| Rule::Literal(Literal::new(Cow::Borrowed(""), Span::default())))
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        for rule in &self.rules {
            rule.get_capturing_groups(count, map, within_variable)?;
        }
        Ok(())
    }

    pub(crate) fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        Ok(Regex::Alternation(RegexAlternation {
            parts: self
                .rules
                .iter()
                .map(|rule| rule.comp(options, state))
                .collect::<Result<_, _>>()?,
        }))
    }

    pub(crate) fn validate(&self, options: &ParseOptions) -> Result<(), ParseError> {
        for rule in &self.rules {
            rule.validate(options)?;
        }
        Ok(())
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

#[cfg_attr(feature = "dbg", derive(Debug))]
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

use std::collections::HashMap;

use crate::{
    compile::{CompileResult, CompileState},
    error::{CompileError, ParseError},
    features::PomskyFeatures,
    options::{CompileOptions, ParseOptions},
    repetition::RegexQuantifier,
    rule::Rule,
    span::Span,
};

#[derive(Clone)]
pub(crate) struct StmtExpr<'i> {
    stmt: Stmt<'i>,
    rule: Rule<'i>,
    pub(crate) span: Span,
}

#[derive(Clone)]
pub(crate) enum Stmt<'i> {
    Enable(BooleanSetting),
    Disable(BooleanSetting),
    Let(Let<'i>),
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum BooleanSetting {
    Lazy,
}

#[derive(Clone)]
pub(crate) struct Let<'i> {
    name: &'i str,
    rule: Rule<'i>,
    pub(crate) name_span: Span,
}

impl<'i> Let<'i> {
    pub(crate) fn new(name: &'i str, rule: Rule<'i>, name_span: Span) -> Self {
        Self { name, rule, name_span }
    }

    pub(crate) fn name(&self) -> &'i str {
        self.name
    }
}

impl<'i> StmtExpr<'i> {
    pub(crate) fn new(stmt: Stmt<'i>, rule: Rule<'i>, span: Span) -> Self {
        Self { stmt, rule, span }
    }

    pub(crate) fn get_capturing_groups(
        &self,
        count: &mut u32,
        map: &'i mut HashMap<String, u32>,
        within_variable: bool,
    ) -> Result<(), CompileError> {
        if let Stmt::Let(l) = &self.stmt {
            l.rule.get_capturing_groups(count, map, true)?;
        }
        self.rule.get_capturing_groups(count, map, within_variable)
    }

    pub(crate) fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c, 'i>,
    ) -> CompileResult<'i> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Lazy;
                let res = self.rule.comp(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
            Stmt::Disable(BooleanSetting::Lazy) => {
                let prev = state.default_quantifier;
                state.default_quantifier = RegexQuantifier::Greedy;
                let res = self.rule.comp(options, state)?;
                state.default_quantifier = prev;
                Ok(res)
            }
            Stmt::Let(r#let) => {
                state.variables.push((r#let.name, &r#let.rule));
                let res = self.rule.comp(options, state)?;
                state.variables.pop();
                Ok(res)
            }
        }
    }

    pub(crate) fn validate(&self, options: &ParseOptions) -> Result<(), ParseError> {
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => {
                options.allowed_features.require(PomskyFeatures::LAZY_MODE, self.span)?;
            }
            Stmt::Disable(_) => {}
            Stmt::Let(l) => {
                options.allowed_features.require(PomskyFeatures::VARIABLES, l.name_span)?;
                l.rule.validate(options)?;
            }
        }

        self.rule.validate(options)
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for StmtExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct DisplayDebug<T>(T);
        impl<T: std::fmt::Display> std::fmt::Debug for DisplayDebug<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        let mut x = f.debug_tuple("StmtExpr");
        let mut x = &mut x;
        match &self.stmt {
            Stmt::Enable(BooleanSetting::Lazy) => x = x.field(&DisplayDebug("enable lazy")),
            Stmt::Disable(BooleanSetting::Lazy) => x = x.field(&DisplayDebug("disable lazy")),
            Stmt::Let(r#let) => x = x.field(r#let),
        }
        x.field(&self.rule).finish()
    }
}

#[cfg(feature = "dbg")]
impl std::fmt::Debug for Let<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {:#?}", self.name, self.rule)
    }
}

use crate::{
    compile::{CompileResult, CompileState},
    diagnose::{CompileErrorKind, Feature},
    options::{CompileOptions, RegexFlavor},
    regex::Regex,
    unicode_set::UnicodeSet,
};

use super::{
    char_class::{RegexCharSet, RegexCompoundCharSet},
    group::RegexGroupKind,
    Compile, Intersection,
};

impl Compile for Intersection {
    fn compile<'c>(
        &'c self,
        options: CompileOptions,
        state: &mut CompileState<'c>,
    ) -> CompileResult {
        // this would be much easier to write with try_reduce, but that's unstable

        let mut rules = self.rules.iter().map(|r| (r.span(), r.compile(options, state)));
        let (first_span, first) = rules.next().expect("Intersection is empty");

        let regex = rules.try_fold(first?, |a, (right_span, b)| match as_sets(a, b?) {
            Ok((left, right)) => left
                .add(right)
                .ok_or_else(|| CompileErrorKind::EmptyIntersection.at(first_span.join(right_span))),
            Err(kind) => Err(kind.at(first_span.join(right_span))),
        })?;

        if let Regex::CompoundCharSet(_) = regex {
            if let RegexFlavor::DotNet | RegexFlavor::Python | RegexFlavor::RE2 = options.flavor {
                return Err(CompileErrorKind::Unsupported(
                    Feature::CharSetIntersection,
                    options.flavor,
                )
                .at(self.span));
            }
        }

        Ok(regex)
    }
}

fn as_sets(a: Regex, b: Regex) -> Result<(RegexCompoundCharSet, RegexCharSet), CompileErrorKind> {
    match (expand_regex(a), expand_regex(b)) {
        (Regex::CompoundCharSet(a), Regex::CharSet(b)) => Ok((a, b)),
        (Regex::CharSet(a), Regex::CharSet(b)) => Ok((RegexCompoundCharSet::new(a), b)),
        _ => Err(CompileErrorKind::BadIntersection),
    }
}

fn expand_regex(r: Regex) -> Regex {
    match r {
        Regex::Literal(ref lit) => {
            let mut chars = lit.chars();
            if let Some(char) = chars.next() {
                if chars.next().is_none() {
                    let mut set = UnicodeSet::new();
                    set.add_char(char);
                    return Regex::CharSet(RegexCharSet::new(set));
                }
            }
            r
        }
        Regex::Group(g) if g.kind == RegexGroupKind::Normal && g.parts.len() == 1 => {
            expand_regex(g.parts.into_iter().next().unwrap())
        }
        _ => r,
    }
}

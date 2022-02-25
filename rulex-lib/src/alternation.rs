use crate::{
    compile::{Compile, CompileResult, CompileState},
    options::CompileOptions,
    Rulex,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Alternation<'i> {
    rules: Vec<Rulex<'i>>,
}

impl<'i> Alternation<'i> {
    fn two(a: Rulex<'i>, b: Rulex<'i>) -> Rulex<'i> {
        Rulex::Alternation(Alternation { rules: vec![a, b] })
    }

    pub fn new_rulex(rules: Vec<Rulex<'i>>) -> Rulex {
        rules
            .into_iter()
            .reduce(reduce)
            .unwrap_or(Rulex::Literal(""))
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

fn reduce<'i>(a: Rulex<'i>, b: Rulex<'i>) -> Rulex<'i> {
    match (a, b) {
        (Rulex::CharClass(a), Rulex::CharClass(b)) => match a.union(b) {
            Ok(c) => Rulex::CharClass(c),
            Err((a, b)) => Alternation::two(Rulex::CharClass(a), Rulex::CharClass(b)),
        },
        (Rulex::Alternation(mut a), Rulex::Alternation(b)) => {
            a.rules.extend(b.rules);
            Rulex::Alternation(a)
        }
        (Rulex::Alternation(mut a), Rulex::CharClass(b))
            if matches!(a.rules.last(), Some(Rulex::CharClass(_))) =>
        {
            match a.rules.pop() {
                Some(Rulex::CharClass(last)) if !last.is_negated() && !b.is_negated() => {
                    match last.union(b) {
                        Ok(c) => a.rules.push(Rulex::CharClass(c)),
                        Err((d, e)) => {
                            a.rules.push(Rulex::CharClass(d));
                            a.rules.push(Rulex::CharClass(e));
                        }
                    }
                }
                Some(last) => {
                    a.rules.push(last);
                    a.rules.push(Rulex::CharClass(b));
                }
                None => unreachable!("We checked in the outer match that a.rules.last() is Some"),
            }
            Rulex::Alternation(a)
        }
        (Rulex::Alternation(mut a), b) => {
            a.rules.push(b);
            Rulex::Alternation(a)
        }
        (a, b) => Alternation::two(a, b),
    }
}

impl Compile for Alternation<'_> {
    fn comp(
        &self,
        options: CompileOptions,
        state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        for rule in &self.rules {
            rule.comp(options, state, buf)?;
            buf.push('|');
        }
        if !self.rules.is_empty() {
            buf.pop().unwrap();
        }
        Ok(())
    }
}

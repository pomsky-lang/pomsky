use crate::{char_class::CharClass, Rulex};

#[derive(Clone, PartialEq, Eq)]
pub struct Alternation<'i> {
    rules: Vec<Rulex<'i>>,
}

impl<'i> Alternation<'i> {
    pub fn new_rulex(rules: Vec<Rulex<'i>>) -> Rulex {
        if rules
            .iter()
            .all(|rule| matches!(rule, Rulex::CharClass(c) if !c.is_negated()))
        {
            let mut cc = CharClass::default();
            for rule in rules {
                match rule {
                    Rulex::CharClass(c) => cc.add_all(c),
                    _ => unreachable!(),
                }
            }
            Rulex::CharClass(cc)
        } else {
            Rulex::Alternation(Alternation { rules })
        }
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

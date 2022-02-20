use core::fmt;

use crate::Rulex;

#[derive(Clone, PartialEq, Eq)]
pub struct Alternation<'i> {
    rules: Vec<Rulex<'i>>,
}

impl<'i> Alternation<'i> {
    pub fn new(rules: Vec<Rulex<'i>>) -> Self {
        Alternation { rules }
    }
}

impl fmt::Debug for Alternation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_tuple("Alternation");
        let mut d = &mut d;
        for rule in &self.rules {
            d = d.field(rule);
        }
        d.finish()
    }
}

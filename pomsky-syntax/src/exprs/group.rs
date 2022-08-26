use crate::Span;

use super::Rule;

/// A group, i.e. sequence of rules. A group is either capturing or
/// non-capturing.
///
/// If it is capturing, it must be wrapped in parentheses, and can have a name.
/// If it is non-capturing, the parentheses can be omitted in same cases.
#[derive(Clone)]
pub struct Group<'i> {
    pub parts: Vec<Rule<'i>>,
    pub capture: Option<Capture<'i>>,
    pub span: Span,
}

impl<'i> Group<'i> {
    pub fn new(parts: Vec<Rule<'i>>, capture: Option<Capture<'i>>, span: Span) -> Self {
        Group { parts, capture, span }
    }

    pub fn set_capture(&mut self, capture: Capture<'i>) {
        self.capture = Some(capture);
    }

    pub fn is_capturing(&self) -> bool {
        self.capture.is_some()
    }
}

#[cfg(feature = "pretty-print")]
impl core::fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.capture {
            Some(Capture { name: Some(name) }) => write!(f, "Group :{name}")?,
            Some(_) => write!(f, "Group :")?,
            None => write!(f, "Group")?,
        }
        if self.parts.is_empty() {
            write!(f, "()")
        } else {
            let mut tup = f.debug_tuple("");
            let mut tup = &mut tup;
            for part in &self.parts {
                tup = tup.field(part);
            }
            tup.finish()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "pretty-print", derive(Debug))]
pub struct Capture<'i> {
    pub name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

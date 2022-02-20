use core::fmt;

use crate::Rulex;

#[derive(Clone, PartialEq, Eq)]
pub struct Group<'i> {
    parts: Vec<Rulex<'i>>,
    capture: Option<Capture<'i>>,
}

impl<'i> Group<'i> {
    pub fn new(parts: Vec<Rulex<'i>>, capture: Option<Capture<'i>>) -> Self {
        Group { parts, capture }
    }

    pub fn set_capture(&mut self, capture: Option<Capture<'i>>) {
        self.capture = capture;
    }
}

impl fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.capture {
            Some(Capture { name: Some(name) }) => write!(f, "Group :{name}")?,
            Some(_) => write!(f, "Group :")?,
            None => write!(f, "Group")?,
        }
        let mut tup = f.debug_tuple("");
        let mut tup = &mut tup;
        for part in &self.parts {
            tup = tup.field(part);
        }
        tup.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture<'i> {
    name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

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

#[cfg(feature = "dbg")]
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

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "dbg", derive(Debug))]
pub struct Capture<'i> {
    name: Option<&'i str>,
}

impl<'i> Capture<'i> {
    pub fn new(name: Option<&'i str>) -> Self {
        Capture { name }
    }
}

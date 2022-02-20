use crate::{
    compile::{Compile, CompileResult, CompileState},
    options::CompileOptions,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Boundary {
    Start,
    Word,
    NotWord,
    End,
}

impl Compile for Boundary {
    fn comp(
        &self,
        _options: CompileOptions,
        _state: &mut CompileState,
        buf: &mut String,
    ) -> CompileResult {
        match self {
            Boundary::Start => buf.push('^'),
            Boundary::Word => buf.push_str("\\b"),
            Boundary::NotWord => buf.push_str("\\B"),
            Boundary::End => buf.push('$'),
        }
        Ok(())
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Boundary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Start => write!(f, "%-"),
            Self::Word => write!(f, "%"),
            Self::NotWord => write!(f, "%!"),
            Self::End => write!(f, "-%"),
        }
    }
}

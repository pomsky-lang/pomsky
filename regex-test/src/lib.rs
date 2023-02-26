mod count;
mod native;
mod sync;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Outcome {
    Success,
    Error(String),
}

pub use sync::RegexTest;

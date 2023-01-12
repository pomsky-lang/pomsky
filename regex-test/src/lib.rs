#[cfg(any(feature = "sync", feature = "async"))]
mod count;

#[cfg(any(feature = "sync", feature = "async"))]
mod native;

#[cfg(any(feature = "sync", feature = "async"))]
pub enum Outcome {
    Success,
    Error(String),
}

// TODO: Still need to compare performance

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "async")]
pub mod r#async;

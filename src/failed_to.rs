use anyhow::{Context, Result};
use std::fmt::Display;

pub trait FailedTo<T, E> {
    fn failed_to<F, D>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> D,
        D: Display;
}

impl<T, E, C> FailedTo<T, E> for C
where
    C: Context<T, E>,
{
    fn failed_to<F, D>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> D,
        D: Display,
    {
        self.with_context(|| format!("failed to {}", f()))
    }
}

pub use anyhow::{Result, anyhow};

#[inline]
pub fn throw<T>(info: &str) -> Result<T> {
    Err(anyhow!("{}", info))
}

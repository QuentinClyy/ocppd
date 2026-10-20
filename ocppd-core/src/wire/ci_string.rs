use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CiString<const N: usize>(String);

#[derive(Debug, Clone, PartialEq)]
pub struct TooLong {
    pub value: String,
    pub max: usize,
}

impl fmt::Display for TooLong {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} is longer than {} characters", self.value, self.max)
    }
}

impl std::error::Error for TooLong {}

impl<const N: usize> CiString<N> {
    pub fn new(value: &str) -> Result<Self, TooLong> {
        Self::try_from(value.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl<const N: usize> TryFrom<String> for CiString<N> {
    type Error = TooLong;

    fn try_from(value: String) -> Result<Self, TooLong> {
        if value.chars().count() > N {
            return Err(TooLong { value, max: N });
        }
        Ok(CiString(value))
    }
}

impl<const N: usize> From<CiString<N>> for String {
    fn from(value: CiString<N>) -> String {
        value.0
    }
}

#[cfg(test)]
#[path = "ci_string.test.rs"]
mod tests;

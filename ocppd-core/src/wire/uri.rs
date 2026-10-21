use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Uri(String);

#[derive(Debug, Clone, PartialEq)]
pub struct InvalidUri(pub String);

impl fmt::Display for InvalidUri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid URI: {:?}", self.0)
    }
}

impl std::error::Error for InvalidUri {}

impl Uri {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Uri {
    type Error = InvalidUri;

    fn try_from(value: String) -> Result<Self, InvalidUri> {
        let scheme = value.split_once(':').map(|(scheme, _)| scheme);
        let valid_scheme = scheme.is_some_and(|scheme| {
            scheme.starts_with(|c: char| c.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        });
        match valid_scheme && !value.chars().any(|c| c.is_whitespace() || c.is_control()) {
            true => Ok(Uri(value)),
            false => Err(InvalidUri(value)),
        }
    }
}

impl From<Uri> for String {
    fn from(value: Uri) -> String {
        value.0
    }
}

#[cfg(test)]
#[path = "uri.test.rs"]
mod tests;

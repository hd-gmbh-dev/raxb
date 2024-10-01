use std::str::FromStr;

use crate::de::XmlDeserializeError;

#[derive(Debug, Clone, Default, Eq, PartialOrd, Ord)]
pub struct ConstStr {
    input_value: String,
    output_value: &'static str,
}

impl From<&'static str> for ConstStr {
    fn from(value: &'static str) -> Self {
        Self {
            input_value: String::default(),
            output_value: value,
        }
    }
}

impl PartialEq for ConstStr {
    fn eq(&self, other: &Self) -> bool {
        if !self.output_value.is_empty() && !other.output_value.is_empty() {
            return self.output_value.eq(other.output_value);
        }
        true
    }
}

impl std::fmt::Display for ConstStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.input_value.fmt(f)
    }
}

impl AsRef<str> for ConstStr {
    fn as_ref(&self) -> &str {
        &self.input_value
    }
}

impl FromStr for ConstStr {
    type Err = XmlDeserializeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            input_value: s.to_string(),
            output_value: "",
        })
    }
}

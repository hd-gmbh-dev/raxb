use std::str::FromStr;

use crate::de::XmlDeserializeError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ConstStr {
    input_value: String,
    output_value: &'static str,
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

pub type XmlTag = &'static [u8];
pub type XmlTargetNs = &'static [u8];

#[derive(Clone)]
pub struct S(pub XmlTag);

impl From<XmlTag> for S {
    fn from(value: XmlTag) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.0))
    }
}

impl std::fmt::Debug for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(self.0))
    }
}

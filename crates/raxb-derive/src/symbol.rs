use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const ROOT: Symbol = Symbol("root");
pub const RAXB: Symbol = Symbol("raxb");
pub const XML: Symbol = Symbol("xml");
pub const NAME: Symbol = Symbol("name");
pub const VALUE: Symbol = Symbol("value");
pub const DEFAULT: Symbol = Symbol("default");
pub const TYPE: Symbol = Symbol("ty");
pub const TNS: Symbol = Symbol("tns");
pub const NS: Symbol = Symbol("ns");
pub const PATH: Symbol = Symbol("path");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, other: &Symbol) -> bool {
        self == other.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

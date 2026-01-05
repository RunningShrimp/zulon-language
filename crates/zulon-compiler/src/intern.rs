use core::fmt;
use std::collections::HashMap;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(pub u32);

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({})", self.0)
    }
}

#[derive(Default)]
pub struct Interner {
    strings: Vec<String>,
    map: HashMap<String, Symbol>,
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }
        let sym = Symbol(self.strings.len() as u32);
        self.strings.push(s.to_owned());
        self.map.insert(s.to_owned(), sym);
        sym
    }

    pub fn resolve(&self, sym: Symbol) -> Option<&str> {
        self.strings.get(sym.0 as usize).map(|s| s.as_str())
    }
}

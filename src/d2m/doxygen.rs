use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

pub type RefID = String;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompoundKind {
    UNKNOWN,
    FILE,
    DIRECTORY,
    NAMESPACE,
    CLASS,
    STRUCT,
    CONCEPT,
    PAGE,
    GROUP,
}

#[derive(Debug)]
pub struct Compound {
    pub name: String,
    pub title: String,
    pub kind: CompoundKind,
}

#[derive(Debug)]
pub struct Registry {
    compounds: HashMap<RefID, Compound>,
}

impl Compound {
    pub fn new() -> Self {
        Self {
            name: String::from("?"),
            title: String::from("?"),
            kind: CompoundKind::UNKNOWN,
        }
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            compounds: HashMap::new()
        }
    }

    pub fn add_compound(&mut self, id: RefID) {
        self.compounds.insert(id, Compound::new());
    }

    pub fn compound_mut(&mut self, id: &RefID) -> Option<&mut Compound> {
        return self.compounds.get_mut(id);
    }

    pub fn compounds(&self) -> Iter<RefID, Compound> {
        return self.compounds.iter();
    }
}

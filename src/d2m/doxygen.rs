use AccessModifier::*;

use std::collections::HashMap;
use std::collections::hash_map::Iter;

pub type RefID = String;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AccessModifier {
    PRIVATE,
    PROTECTED,
    PUBLIC,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub qualified_name: String,
    pub return_type: String,
    pub template_parameters: String,
    pub arguments: String,
    pub definition: String,
    pub access: AccessModifier,
    pub is_static: bool,
    pub is_const: bool,
    pub is_inline: bool,
    pub is_noexcept: bool,
    pub is_virtual: bool,
    pub is_explicit: bool,
    pub is_member: bool,
}

impl Function {
    pub fn new() -> Self {
        Self {
            name: String::from("?"),
            qualified_name: String::from("?"),
            return_type: String::from("?"),
            template_parameters: String::from("?"),
            arguments: String::from("?"),
            definition: String::from("?"),
            access: PRIVATE,
            is_static: false,
            is_const: false,
            is_inline: false,
            is_noexcept: false,
            is_virtual: false,
            is_explicit: false,
            is_member: false,
        }
    }
}

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

impl Compound {
    pub fn new() -> Self {
        Self {
            name: String::from("?"),
            title: String::from("?"),
            kind: CompoundKind::UNKNOWN,
        }
    }
}

#[derive(Debug)]
pub struct Registry {
    compounds: HashMap<RefID, Compound>,
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

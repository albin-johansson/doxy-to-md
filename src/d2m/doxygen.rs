use AccessModifier::*;

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

pub type RefID = String;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AccessModifier {
    PRIVATE,
    PROTECTED,
    PUBLIC,
}

impl fmt::Display for AccessModifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            PRIVATE => "private",
            PROTECTED => "protected",
            PUBLIC => "public"
        })
    }
}

impl FromStr for AccessModifier {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(PRIVATE),
            "protected" => Ok(PROTECTED),
            "public" => Ok(PUBLIC),
            _ => Err("Unsupported access modifier string!")
        }
    }
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub definition: String,
    pub access: AccessModifier,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_mutable: bool,
}

impl Variable {
    pub fn new() -> Self {
        Self {
            name: String::from("?"),
            definition: String::from("?"),
            access: PRIVATE,
            is_static: false,
            is_constexpr: false,
            is_mutable: false,
        }
    }
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
    pub fn new(is_member: bool) -> Self {
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
            is_member,
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub unqualified_name: String,
    pub is_struct: bool,
}

impl Class {
    pub fn new(is_struct: bool) -> Self {
        Self {
            unqualified_name: String::from("?"),
            is_struct,
        }
    }
}

#[derive(Debug)]
pub struct Define {
    pub name: String,
}

impl Define {
    pub fn new() -> Self {
        Self {
            name: String::from("?")
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
    pub groups: Vec<RefID>,
    pub namespaces: Vec<RefID>,
    pub classes: Vec<RefID>,
    pub functions: Vec<RefID>,
    pub variables: Vec<RefID>,
    pub defines: Vec<RefID>,
}

impl Compound {
    pub fn new() -> Self {
        Self {
            name: String::from("?"),
            title: String::from("?"),
            kind: CompoundKind::UNKNOWN,
            groups: Vec::new(),
            namespaces: Vec::new(),
            classes: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            defines: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Registry {
    pub compounds: HashMap<RefID, Compound>,
    pub classes: HashMap<RefID, Class>,
    pub functions: HashMap<RefID, Function>,
    pub variables: HashMap<RefID, Variable>,
    pub defines: HashMap<RefID, Define>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            compounds: HashMap::new(),
            classes: HashMap::new(),
            functions: HashMap::new(),
            variables: HashMap::new(),
            defines: HashMap::new(),
        }
    }
}

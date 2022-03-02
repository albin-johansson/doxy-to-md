use crate::d2m::doxygen::RefID;
use crate::d2m::doxygen::CompoundKind;
use crate::d2m::doxygen::CompoundKind::*;
use crate::d2m::doxygen::Registry;

use std::{fs, path};
use std::path::PathBuf;
use minidom::{Element, NSChoice};

fn parse_member_declaration(registry: &mut Registry,
                            element: &Element,
                            compound_kind: CompoundKind) {
    let ref_id: RefID = element.attr("refid").unwrap().to_owned();

    match element.attr("kind") {
        Some("define") => {}
        Some("friend") => registry.add_function(ref_id, compound_kind == CLASS || compound_kind == STRUCT),
        Some("typedef") => {}
        Some("variable") => {}
        Some("function") => {}
        Some("enum") => {}
        Some("enumvalue") => {}
        Some(x) => panic!("Encountered unsupported member type: {}", x),
        _ => panic!("Member declaration in index has no kind attribute!")
    };
}

fn parse_compound_declaration(registry: &mut Registry, element: &Element) {
    let ref_id: RefID = element.attr("refid").unwrap().to_owned();

    registry.add_compound(ref_id.to_owned());
    let mut compound = registry.compound_mut(&ref_id).unwrap();

    if let Some(name_elem) = element.get_child("name", NSChoice::Any) {
        compound.name = name_elem.text();
    }

    compound.kind = match element.attr("kind") {
        Some("file") => FILE,
        Some("dir") => DIRECTORY,
        Some("namespace") => NAMESPACE,
        Some("class") => CLASS,
        Some("struct") => STRUCT,
        Some("concept") => CONCEPT,
        Some("page") => PAGE,
        Some("group") => GROUP,
        Some(x) => panic!("Encountered unsupported compound type: {}", x),
        _ => panic!("Compound declaration in index has no kind attribute!")
    };

    let kind = compound.kind;
    for child in element.children() {
        if child.name() == "member" {
            parse_member_declaration(registry, &child, kind);
        }
    }
}

fn parse_index_file(input_dir: &PathBuf) -> Registry {
    let index_file = input_dir.join("index.xml");
    println!("Parsing index file {}", index_file.display());

    let raw_contents = fs::read_to_string(&index_file).unwrap();
    let root_element: minidom::Element = raw_contents.parse().unwrap();

    let mut registry = Registry::new();

    for child in root_element.children() {
        match child.name() {
            "compound" => parse_compound_declaration(&mut registry, &child),
            name => println!("Ignoring element with name: {}", name)
        }
    }

    return registry;
}

pub fn parse_xml(input_dir: &PathBuf) -> Registry {
    println!("Parsing XML input...");
    let mut registry = parse_index_file(input_dir);

    // TODO

    return registry;
}

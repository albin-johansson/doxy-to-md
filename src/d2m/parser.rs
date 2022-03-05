use crate::d2m::doxygen::*;
use crate::d2m::doxygen::CompoundKind::*;
use crate::d2m::doxygen::AccessModifier::*;

use std::str::FromStr;
use std::borrow::BorrowMut;
use std::fs;
use std::path::PathBuf;
use minidom::{Element, NSChoice};

fn parse_xml_file(path: &PathBuf) -> Element
{
    let raw_contents = fs::read_to_string(&path).unwrap();
    let root_element: minidom::Element = raw_contents.parse().unwrap();
    return root_element;
}

fn parse_compound_kind(kind: Option<&str>) -> CompoundKind
{
    return match kind {
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
}

fn parse_brief_docs(elem: &Element) -> Vec<String>
{
    assert_eq!(elem.name(), "briefdescription");

    let mut docs = Vec::new();

    for child in elem.children() {
        if child.name() == "para" {
            docs.push(child.text());
        }
    }

    return docs;
}

fn parse_detailed_docs(elem: &Element) -> Vec<String>
{
    assert_eq!(elem.name(), "detaileddescription");

    let mut docs = Vec::new();

    for child in elem.children() {
        if child.name() == "para" {
            if child.has_child("parameterlist", NSChoice::Any) {
                // TODO
            } else {
                docs.push(child.text().replace('\n', " "));
            }
        }
    }

    return docs;
}

fn parse_template_args(elem: &Element) -> Vec<String>
{
    let mut args = Vec::new();

    for param in elem.children() {
        if param.name() == "param" {
            if let Some(name) = param.get_child("type", NSChoice::Any) {
                args.push(name.text());
            }
        }
    }

    return args;
}

fn remove_redundant_const_from_function_parameters(func: &mut Function)
{
    let uses_trailing_return = func.return_type == "auto";

    let mut head = String::new();
    let mut tail = String::new();
    if uses_trailing_return {
        let mut separated = func.args.split("->");
        head = separated.next().unwrap().to_owned();
        tail = separated.next().unwrap().to_owned();
    }

    let mut new_args = String::with_capacity(func.args.len());
    let mut first = true;

    for arg in head.split(",") {
        let is_pointer = arg.contains("*") || arg.contains("&");

        if !first {
            new_args.push(',');
        }

        if !is_pointer && arg.contains("const") {
            new_args.push_str(arg.replace("const ", "").as_str());
        } else {
            new_args.push_str(arg);
        }

        first = false;
    }

    if uses_trailing_return {
        new_args.push_str("->");
        new_args.push_str(tail.as_str());
    }

    // new_args.remove(new_args.len() - 1);

    func.args = new_args;
}

fn simplify_function_noexcept_specifier(func: &mut Function)
{
    // TODO
}

fn parse_function_definition(elem: &Element, func: &mut Function)
{
    func.is_static = elem.attr("static").unwrap() == "yes";
    func.is_const = elem.attr("const").unwrap() == "yes";
    func.is_explicit = elem.attr("explicit").unwrap() == "yes";
    func.is_inline = elem.attr("inline").unwrap() == "yes";
    func.is_virtual = elem.attr("virt").unwrap() != "non-virtual";
    func.is_noexcept = elem.attr("const").unwrap_or("no") == "yes";

    func.access = AccessModifier::from_str(elem.attr("prot").unwrap()).unwrap();

    func.name = elem.get_child("name", NSChoice::Any).unwrap().text();
    if let Some(qname) = elem.get_child("qualifiedname", NSChoice::Any) {
        func.qualified_name = qname.text();
    }

    func.definition = elem.get_child("definition", NSChoice::Any).unwrap().text();
    func.return_type = elem.get_child("type", NSChoice::Any).unwrap().text();

    func.args = elem.get_child("argsstring", NSChoice::Any).unwrap().text();

    if let Some(args) = elem.get_child("templateparamlist", NSChoice::Any) {
        func.template_args = parse_template_args(args);
    }

    if let Some(brief) = elem.get_child("briefdescription", NSChoice::Any) {
        func.brief_docs = parse_brief_docs(&brief);
    }

    if let Some(details) = elem.get_child("detaileddescription", NSChoice::Any) {
        func.detailed_docs = parse_detailed_docs(&details);
    }

    remove_redundant_const_from_function_parameters(func);
    simplify_function_noexcept_specifier(func);
    // _parse_documentation(func, node)
}

fn parse_variable_definition(elem: &Element, var: &mut Variable)
{
    var.access = AccessModifier::from_str(elem.attr("prot").unwrap()).unwrap();

    var.is_static = elem.attr("static").unwrap() == "yes";
    var.is_mutable = elem.attr("mutable").unwrap() == "yes";
    var.is_constexpr = elem.attr("constexpr").unwrap_or("no") == "yes";

    var.definition = elem.get_child("definition", NSChoice::Any).unwrap().text();
}

fn parse_compound_definition(element: &Element, registry: &mut Registry)
{
    let ref_id = element.attr("id").unwrap();

    let mut compound = registry.compounds.get_mut(ref_id).unwrap();

    if let Some(title) = element.get_child("title", NSChoice::Any) {
        compound.title = title.text();
    }

    for elem in element.children() {
        match elem.name() {
            "compoundname" => (), // Do nothing
            "title" => (),        // Do nothing
            "briefdescription" => compound.brief_docs = parse_brief_docs(elem),
            "detaileddescription" => compound.detailed_docs = parse_detailed_docs(elem),
            "innergroup" => {
                if let Some(id) = elem.attr("refid") {
                    compound.groups.push(id.to_owned());
                }
            }
            "innerclass" => {
                if let Some(id) = elem.attr("refid") {
                    compound.classes.push(id.to_owned());
                }
            }
            "innernamespace" => {
                if let Some(id) = elem.attr("refid") {
                    compound.namespaces.push(id.to_owned());
                }
            }
            "sectiondef" => {
                for child in elem.children() {
                    if child.name() == "memberdef" {
                        let child_id: RefID = child.attr("id").unwrap().to_owned();
                        let kind = child.attr("kind").unwrap();
                        if kind == "function" {
                            let mut func = registry.functions.get_mut(&child_id).unwrap();
                            parse_function_definition(child, &mut func);
                        }
                    }
                }
            }
            "memberdef" => {
                let ref_id: RefID = elem.attr("id").unwrap().to_owned();
                let kind = elem.attr("kind").unwrap();

                if kind == "function" {
                    let mut func = registry.functions.get_mut(&ref_id).unwrap();
                    parse_function_definition(elem, &mut func);
                } else if kind == "variable" {
                    let mut var = registry.variables.get_mut(&ref_id).unwrap();
                    parse_variable_definition(elem, &mut var);
                }

                // TODO enum
            }
            "templateparamlist" => {
                if let Some(class) = registry.classes.get_mut(ref_id) {
                    class.template_args = parse_template_args(elem);
                }
            }
            _ => ()
        }
    }
}

fn parse_generic_file(file_path: &PathBuf, registry: &mut Registry)
{
    if file_path.is_file() && file_path.file_name().unwrap() != "index.xml" {
        println!("Parsing file {}", file_path.display());

        let root_element = parse_xml_file(&file_path);
        for child in root_element.children() {
            match child.name() {
                "compounddef" => parse_compound_definition(child, registry),
                _name => () //println!("Ignoring element with name: {}", name)
            }
        }
    }
}

fn parse_member_declaration(registry: &mut Registry, element: &Element, parent_id: &RefID)
{
    let parent = registry.compounds.get_mut(parent_id).unwrap();
    let ref_id: RefID = element.attr("refid").unwrap().to_owned();

    match element.attr("kind") {
        Some("define") => {
            registry.defines.insert(ref_id.to_owned(), Define::new());
            parent.defines.push(ref_id.to_owned());
        }
        Some("friend") => {}
        Some("typedef") => {}
        Some("variable") => {
            registry.variables.insert(ref_id.to_owned(), Variable::new());
            parent.variables.push(ref_id.to_owned());
        }
        Some("function") => {
            registry.functions.insert(ref_id.to_owned(),
                                      Function::new(parent.kind == CLASS || parent.kind == STRUCT));
            parent.functions.push(ref_id.to_owned());
        }
        Some("enum") => {}
        Some("enumvalue") => {}
        Some(x) => panic!("Encountered unsupported member type: {}", x),
        _ => panic!("Member declaration in index has no kind attribute!")
    };
}

fn parse_class_declaration(registry: &mut Registry, ref_id: &RefID, name: &String, is_struct: bool)
{
    let reg = registry.borrow_mut();
    reg.classes.insert(ref_id.to_owned(), Class::new(is_struct));

    let class = reg.classes.get_mut(ref_id).unwrap();
    class.unqualified_name = name.split("::").last().unwrap().to_owned();
}

fn parse_compound_declaration(registry: &mut Registry, element: &Element)
{
    let ref_id: RefID = element.attr("refid").unwrap().to_owned();

    let mut name = String::from("?");
    if let Some(name_elem) = element.get_child("name", NSChoice::Any) {
        name = name_elem.text();
    }

    let kind = parse_compound_kind(element.attr("kind"));
    match &kind {
        CLASS => parse_class_declaration(registry, &ref_id, &name, false),
        STRUCT => parse_class_declaration(registry, &ref_id, &name, true),
        _ => ()
    }

    registry.compounds.insert(ref_id.to_owned(), Compound::new());
    let mut compound = registry.compounds.get_mut(&ref_id).unwrap();
    compound.kind = kind;
    compound.name = name;

    for child in element.children() {
        if child.name() == "member" {
            parse_member_declaration(registry, child, &ref_id);
        }
    }
}

fn parse_index_file(input_dir: &PathBuf) -> Registry
{
    let index_file = input_dir.join("index.xml");
    println!("Parsing index file {}", index_file.display());

    let root_element = parse_xml_file(&index_file);
    let mut registry = Registry::new();

    for child in root_element.children() {
        match child.name() {
            "compound" => parse_compound_declaration(&mut registry, child),
            _name => () //println!("Ignoring element with name: {}", name)
        }
    }

    return registry;
}

pub fn parse_xml(input_dir: &PathBuf) -> Registry
{
    println!("Parsing XML input...");
    let mut registry = parse_index_file(input_dir);

    for e in fs::read_dir(input_dir).unwrap() {
        match e {
            Ok(entry) => parse_generic_file(&entry.path(), &mut registry),
            Err(x) => println!("Error encountered when iterating input directory: {}", x)
        }
    }

    return registry;
}

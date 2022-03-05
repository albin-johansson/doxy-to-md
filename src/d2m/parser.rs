use std::borrow::BorrowMut;
use std::fs;
use std::iter;
use std::path::PathBuf;
use std::str::FromStr;

use minidom::Element;
use minidom::NSChoice::Any as AnyNS;

use crate::d2m::doxygen::CompoundKind::*;
use crate::d2m::doxygen::*;

fn parse_xml_file(path: &PathBuf) -> Element
{
  let raw_contents = fs::read_to_string(&path).unwrap();

  let root_element: minidom::Element = raw_contents.parse().unwrap();
  return root_element;
}

fn parse_text(root: &Element) -> String
{
  let mut content = String::new();

  for node in root.nodes() {
    // This is the base case for the recursion
    if let Some(text) = node.as_text() {
      if !text.is_empty() {
        content += text.trim();
      }
    } else if let Some(elem) = node.as_element() {
      match elem.name() {
        "para" => {
          content += parse_text(elem).as_str();
        }
        "computeroutput" => {
          content += format!(" `{}` ", parse_text(elem)).as_str();
        }
        "itemizedlist" => {
          content += format!("\n{}", parse_text(elem)).as_str();
        }
        "listitem" => {
          content += format!("* {}\n", parse_text(elem)).as_str();
        }
        "ref" => {
          // TODO use
          let _referenced_id = elem.attr("refid").unwrap();
          let _referenced_kind = elem.attr("kindref").unwrap();
          content += format!("[{}](foo.md)", parse_text(elem)).as_str();
        }
        _ => ()
      }
    }
  }

  return content;
}

fn parse_comment(elem: &Element) -> Comment
{
  let mut comment = Comment::new();

  if let Some(brief) = elem.get_child("briefdescription", AnyNS) {
    if let Some(para) = brief.get_child("para", AnyNS) {
      comment.brief.push(parse_text(para));
    }
  }

  if let Some(detailed) = elem.get_child("detaileddescription", AnyNS) {
    for para in detailed.children().filter(|x| x.is("para", AnyNS)) {
      comment.details.push(parse_text(para));
    }

    // for para in detailed.children().filter(|x| x.is("para", AnyNS)) {
    //   if let Some(list) = para.get_child("parameterlist", AnyNS) {
    //     let kind = list.attr("kind").unwrap();
    //     if kind == "param" {
    //       for item in list.children().filter(|x| x.is("parameteritem", AnyNS)) {
    //         let mut name = None;
    //
    //         if let Some(list) = item.get_child("parameternamelist", AnyNS) {
    //           if let Some(parameter_name) = list.get_child("parametername", AnyNS) {
    //             name = Some(parameter_name.text());
    //           }
    //         }
    //
    //         if let Some(desc) = item.get_child("parameterdescription", AnyNS) {
    //           let mut para = parse_para(desc);
    //           comment
    //               .parameters
    //               .insert(name.unwrap(), String::from(para.first().unwrap()));
    //         }
    //       }
    //     }
    //   }
    //
    //   if let Some(simple_section) = para.get_child("simplesect", AnyNS) {
    //     let kind = simple_section.attr("kind").unwrap();
    //     if kind == "return" {
    //       if let Some(para) = simple_section.get_child("para", AnyNS) {
    //         comment.returns = para.text();
    //       }
    //     }
    //   }
    // }
  }

  return comment;
}

fn parse_template_args(elem: &Element) -> Vec<String>
{
  let mut args = Vec::new();

  for param in elem.children() {
    if param.name() == "param" {
      if let Some(name) = param.get_child("type", AnyNS) {
        args.push(name.text());
      }
    }
  }

  return args;
}

fn remove_redundant_const_from_function_parameters(func: &mut Function)
{
  if func.parameter_names.is_empty() {
    return;  // No need to process functions with zero arguments
  }

  let uses_trailing_return = func.return_type == "auto";

  let head;
  let tail;
  if uses_trailing_return {
    let mut separated = func.args.split("->");
    head = separated.next().unwrap().to_owned();
    tail = separated.next().unwrap().to_owned();
  } else {
    head = func.args.to_owned();
    tail = String::new();
  }

  let mut new_args = String::with_capacity(func.args.len());
  let mut first = true;

  let alignment_offset = func.return_type.len() + func.name.len() + 1;

  for arg in head.split(",").filter(|x| !x.is_empty()) {
    let is_pointer = arg.contains("*") || arg.contains("&");

    if !first {
      new_args += ",\n";
      new_args += iter::repeat(" ").take(alignment_offset).collect::<String>().as_str();
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

  func.args = new_args;
}

fn simplify_function_noexcept_specifier(_func: &mut Function)
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

  func.name = elem.get_child("name", AnyNS).unwrap().text();
  if let Some(qname) = elem.get_child("qualifiedname", AnyNS) {
    func.qualified_name = qname.text();
  }

  func.definition = elem.get_child("definition", AnyNS).unwrap().text();
  func.return_type = elem.get_child("type", AnyNS).unwrap().text();

  func.args = elem.get_child("argsstring", AnyNS).unwrap().text();

  if let Some(args) = elem.get_child("templateparamlist", AnyNS) {
    func.template_args = parse_template_args(args);
  }

  // Parse parameter names, even if they may be undocumented
  for child in elem.children().filter(|x| x.is("param", AnyNS)) {
    if let Some(name) = child.get_child("declname", AnyNS) {
      let text = name.text();

      // Information is occasionally duplicated, such as in namespace and group files
      if !func.parameter_names.contains(&text) {
        func.parameter_names.push(text);
      }
    }
  }

  func.docs = parse_comment(&elem);

  remove_redundant_const_from_function_parameters(func);
  simplify_function_noexcept_specifier(func);
}

fn parse_variable_definition(elem: &Element, var: &mut Variable)
{
  var.access = AccessModifier::from_str(elem.attr("prot").unwrap()).unwrap();

  var.is_static = elem.attr("static").unwrap() == "yes";
  var.is_mutable = elem.attr("mutable").unwrap() == "yes";
  var.is_constexpr = elem.attr("constexpr").unwrap_or("no") == "yes";

  var.definition = elem.get_child("definition", AnyNS).unwrap().text();
}

fn parse_compound_definition(element: &Element, registry: &mut Registry)
{
  let ref_id = element.attr("id").unwrap();

  let mut compound = registry.compounds.get_mut(ref_id).unwrap();

  if let Some(title) = element.get_child("title", AnyNS) {
    compound.title = title.text();
  }

  for elem in element.children() {
    match elem.name() {
      "compoundname" => (), // Do nothing
      "title" => (),        // Do nothing
      // "briefdescription" => compound.brief_docs = parse_para(elem),
      // "detaileddescription" => compound.detailed_docs = parse_para(elem),
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
      _ => (),
    }
  }
}

fn parse_generic_file(file_path: &PathBuf, registry: &mut Registry)
{
  if file_path.is_file() && file_path.file_name().unwrap() != "index.xml" {
    // println!("Parsing file {}", file_path.display());

    let root_element = parse_xml_file(&file_path);
    for child in root_element.children() {
      match child.name() {
        "compounddef" => parse_compound_definition(child, registry),
        _name => (), //println!("Ignoring element with name: {}", name)
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
      registry
          .variables
          .insert(ref_id.to_owned(), Variable::new());
      parent.variables.push(ref_id.to_owned());
    }
    Some("function") => {
      registry.functions.insert(
        ref_id.to_owned(),
        Function::new(parent.kind == CLASS || parent.kind == STRUCT),
      );
      parent.functions.push(ref_id.to_owned());
    }
    Some("enum") => {}
    Some("enumvalue") => {}
    Some(x) => panic!("Encountered unsupported member type: {}", x),
    _ => panic!("Member declaration in index has no kind attribute!"),
  };
}

fn parse_class_declaration(registry: &mut Registry,
                           ref_id: &RefID,
                           name: &String,
                           is_struct: bool)
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
  if let Some(name_elem) = element.get_child("name", AnyNS) {
    name = name_elem.text();
  }

  let kind = CompoundKind::from_str(element.attr("kind").unwrap()).unwrap();
  match kind {
    CLASS => parse_class_declaration(registry, &ref_id, &name, false),
    STRUCT => parse_class_declaration(registry, &ref_id, &name, true),
    _ => (),
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
  // println!("Parsing index file {}", index_file.display());

  let root_element = parse_xml_file(&index_file);
  let mut registry = Registry::new();

  for child in root_element.children() {
    match child.name() {
      "compound" => parse_compound_declaration(&mut registry, child),
      _name => (), //println!("Ignoring element with name: {}", name)
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
      Err(x) => println!("Error encountered when iterating input directory: {}", x),
    }
  }

  return registry;
}

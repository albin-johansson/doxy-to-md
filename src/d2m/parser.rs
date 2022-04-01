use std::collections::HashMap;
use std::fs;
use std::iter;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

use minidom::Element;
use minidom::NSChoice::Any as AnyNS;
use lazy_static::lazy_static;
use regex::Regex;

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
        "para" => content += parse_text(elem).as_str(),
        "computeroutput" => content += format!(" `{}` ", parse_text(elem)).as_str(),
        "itemizedlist" => content += format!("\n{}", parse_text(elem)).as_str(),
        "listitem" => content += format!("* {}\n", parse_text(elem)).as_str(),
        "ref" => {
          // TODO use
          let _referenced_id = elem.attr("refid").unwrap();
          let _referenced_kind = elem.attr("kindref").unwrap();
          // content += format!("[{}](foo.md)", parse_text(elem)).as_str();
          content += format!("{}", parse_text(elem)).as_str();
        }
        _ => ()
      }
    }
  }

  return content;
}

fn parse_parameter_list(elem: &Element) -> HashMap<String, String>
{
  assert_eq!(elem.name(), "parameterlist");
  let mut entries = HashMap::new();

  for item in elem.children().filter(|e| e.is("parameteritem", AnyNS)) {
    let list = item.get_child("parameternamelist", AnyNS).unwrap();

    let name_elem = list.get_child("parametername", AnyNS).unwrap();
    let name = parse_text(name_elem);

    let desc_elem = item.get_child("parameterdescription", AnyNS).unwrap();
    let desc = parse_text(desc_elem);

    entries.insert(name, desc);
  }

  return entries;
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
    for child in detailed.children() {
      match child.name() {
        "para" => {
          // The parse_text function ignores parameter lists and sections
          comment.details.push(parse_text(child));

          if let Some(parameter_list) = child.get_child("parameterlist", AnyNS) {
            match parameter_list.attr("kind").unwrap() {
              "param" => {
                assert!(comment.parameters.is_empty());
                comment.parameters = parse_parameter_list(parameter_list);
              }
              "exception" => {
                assert!(comment.exceptions.is_empty());
                comment.exceptions = parse_parameter_list(parameter_list);
              }
              "templateparam" => {
                assert!(comment.template_parameters.is_empty());
                comment.template_parameters = parse_parameter_list(parameter_list);
              }
              kind => println!("Ignoring parameter list of type '{}'", kind)
            }
          }

          if let Some(simple_section) = child.get_child("simplesect", AnyNS) {
            match simple_section.attr("kind").unwrap() {
              "return" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.returns = parse_text(para);
                }
              }
              "note" | "remark" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.notes.push(parse_text(para));
                }
              }
              "see" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.see_also.push(parse_text(para));
                }
              }
              "warning" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.warnings.push(parse_text(para));
                }
              }
              "pre" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.pre_conditions.push(parse_text(para));
                }
              }
              "post" => {
                if let Some(para) = simple_section.get_child("para", AnyNS) {
                  comment.post_conditions.push(parse_text(para));
                }
              }
              kind => println!("Ignoring simple section of type '{}'", kind),
            }
          }
        }
        tag => println!("Ignoring child of detailed description with tag '{}'", tag)
      }
    }
  }

  return comment;
}

fn parse_template_args(elem: &Element) -> Vec<String>
{
  let mut args = Vec::new();

  for param in elem.children().filter(|e| e.is("param", AnyNS)) {
    let type_elem = param.get_child("type", AnyNS).unwrap();
    args.push(type_elem.text());
  }

  return args;
}

fn remove_redundant_const_from_function_parameters(func: &mut Function)
{
  if func.parameter_names.is_empty() {
    return;  // No need to process functions with zero arguments
  }

  let uses_trailing_return = func.return_type == "auto";

  let (head, tail) = if uses_trailing_return {
    let mut separated = func.args.split("->");
    (separated.next().unwrap().to_owned(), separated.next().unwrap().to_owned())
  } else {
    (func.args.to_owned(), String::new())
  };

  let mut new_args = String::with_capacity(func.args.len());
  let mut first = true;

  let align_offset = func.return_type.len()
      + func.name.len()
      + if func.is_static { 7 } else { 0 }
      + if func.is_explicit { 9 } else { 0 }
      + if func.return_type.is_empty() { 0 } else { 1 };

  for arg in head.split(",").filter(|s| !s.is_empty()) {
    let is_pointer = arg.contains("*") || arg.contains("&");

    if !first {
      new_args += ",";
      if !arg.contains("<") && !arg.contains(">") {
        new_args += "\n";
        new_args += iter::repeat(" ").take(align_offset).collect::<String>().as_str();
      }
    }

    if !is_pointer && arg.contains("const") {
      new_args += arg.replace("const ", "").as_str();
    } else {
      new_args += arg;
    }

    first = false;
  }

  if uses_trailing_return {
    new_args.push_str("->");
    new_args.push_str(tail.as_str());
  }

  func.args = new_args;
}

fn simplify_function_noexcept_specifier(func: &mut Function)
{
  lazy_static! {
    static ref RE: Regex = Regex::new(r"(?P<a>noexcept)(.*)").unwrap();
  }

  if func.args.contains("noexcept(") {
    func.args = RE.replace_all(&func.args, "$a(...)").to_string();
  }
}

fn parse_function_definition(elem: &Element, func: &mut Function)
{
  func.access = AccessModifier::from_str(elem.attr("prot").unwrap()).unwrap();

  func.is_static = elem.attr("static").unwrap() == "yes";
  func.is_const = elem.attr("const").unwrap() == "yes";
  func.is_explicit = elem.attr("explicit").unwrap() == "yes";
  func.is_inline = elem.attr("inline").unwrap() == "yes";
  func.is_virtual = elem.attr("virt").unwrap() != "non-virtual";
  func.is_noexcept = elem.attr("const").unwrap_or("no") == "yes";

  func.name = elem.get_child("name", AnyNS).unwrap().text();
  func.definition = elem.get_child("definition", AnyNS).unwrap().text();
  func.return_type = elem.get_child("type", AnyNS).unwrap().text();
  func.args = elem.get_child("argsstring", AnyNS).unwrap().text();

  if let Some(qname) = elem.get_child("qualifiedname", AnyNS) {
    func.qualified_name = qname.text();
  }

  if let Some(args) = elem.get_child("templateparamlist", AnyNS) {
    func.template_args = parse_template_args(args);
  }

  // Parse parameter names, even if they may be undocumented
  for child in elem.children().filter(|e| e.is("param", AnyNS)) {
    if let Some(decl_name) = child.get_child("declname", AnyNS) {
      let name = decl_name.text();

      // Information is occasionally duplicated, such as in namespace and group files
      if !func.parameter_names.contains(&name) {
        func.parameter_names.push(name);
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

  var.name = elem.get_child("name", AnyNS).unwrap().text();
  var.qualified_name = elem.get_child("qualifiedname", AnyNS).unwrap().text();
  var.definition = elem.get_child("definition", AnyNS).unwrap().text();

  var.docs = parse_comment(elem);
}

fn parse_enum_definition(elem: &Element, e: &mut Enum)
{
  e.name = elem.get_child("name", AnyNS).unwrap().text();
  e.qualified_name = elem.get_child("qualifiedname", AnyNS).unwrap().text();
  e.is_scoped = elem.attr("strong").unwrap() == "yes";

  e.docs = parse_comment(elem);

  for value_elem in elem.children().filter(|c| c.is("enumvalue", AnyNS)) {
    let mut value = EnumValue::new();

    value.name = value_elem.get_child("name", AnyNS).unwrap().text();

    if let Some(initializer) = value_elem.get_child("initializer", AnyNS) {
      value.initializer = initializer.text().replace("= ", "");
    }

    value.docs = parse_comment(value_elem);

    e.values.push(value);
  }
}

fn parse_compound_definition(element: &Element, registry: &mut Registry)
{
  let kind = element.attr("kind").unwrap();

  if kind == "file" || kind == "namespace" {
    return;
  }

  let compound_id = element.attr("id").unwrap();
  let mut compound = registry.compounds.get_mut(compound_id).unwrap();

  for elem in element.children() {
    match elem.name() {
      "title" => compound.title = parse_text(elem),
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
        for member in elem.children().filter(|e| e.is("memberdef", AnyNS)) {
          let member_id: RefID = member.attr("id").unwrap().to_owned();

          match member.attr("kind").unwrap() {
            "function" => {
              let mut func = registry.functions.get_mut(&member_id).unwrap();
              parse_function_definition(member, &mut func);
            }
            "variable" => {
              let mut var = registry.variables.get_mut(&member_id).unwrap();
              parse_variable_definition(member, &mut var);
            }
            "enum" => {
              let mut e = registry.enums.get_mut(&member_id).unwrap();
              parse_enum_definition(member, &mut e);
            }
            _ => ()
          }
        }
      }
      "templateparamlist" => {
        if let Some(class) = registry.classes.get_mut(compound_id) {
          class.template_args = parse_template_args(elem);
        }
      }
      _ => ()
    }
  }
}

fn parse_generic_file(file_path: &PathBuf, registry: &mut Registry)
{
  if file_path.is_file()
      && file_path.extension().unwrap() == "xml"
      && file_path.file_name().unwrap() != "index.xml" {
    println!("Parsing file {}", file_path.display());

    let root_element = parse_xml_file(&file_path);
    for elem in root_element.children().filter(|e| e.is("compounddef", AnyNS)) {
      parse_compound_definition(elem, registry);
    }
  }
}

fn parse_member_declaration(registry: &mut Registry, element: &Element, parent_id: &RefID)
{
  let parent = registry.compounds.get_mut(parent_id).unwrap();
  let member_id = element.attr("refid").unwrap().to_owned();

  match element.attr("kind").unwrap() {
    "define" => {
      registry.defines.insert(member_id.to_owned(), Define::new());
      parent.defines.push(member_id.to_owned());
    }
    "friend" => {}
    "typedef" => {}
    "variable" => {
      registry.variables.insert(member_id.to_owned(), Variable::new());
      parent.variables.push(member_id.to_owned());
    }
    "function" => {
      registry.functions.insert(member_id.to_owned(),
                                Function::new(parent.kind == CLASS || parent.kind == STRUCT));
      parent.functions.push(member_id.to_owned());
    }
    "enum" => {
      registry.enums.insert(member_id.to_owned(), Enum::new());
      parent.enums.push(member_id.to_owned());
    }
    "enumvalue" => {
      registry.enum_values.insert(member_id.to_owned(), EnumValue::new());
      parent.enum_values.push(member_id.to_owned());
    }
    kind => println!("Ignoring member declaration of type '{}'", kind),
  };
}

fn parse_class_declaration(registry: &mut Registry,
                           ref_id: &RefID,
                           name: &String,
                           clazz: Class)
{
  registry.classes.insert(ref_id.to_owned(), clazz);

  let class = registry.classes.get_mut(ref_id).unwrap();
  class.unqualified_name = name.split("::").last().unwrap().to_owned();
}

fn parse_compound_declaration(registry: &mut Registry, element: &Element)
{
  let compound_id = element.attr("refid").unwrap().to_owned();

  let name = match element.get_child("name", AnyNS) {
    Some(name) => name.text(),
    None => String::from("?")
  };

  let kind = CompoundKind::from_str(element.attr("kind").unwrap()).unwrap();

  match kind {
    CLASS => parse_class_declaration(registry, &compound_id, &name, Class::new()),
    STRUCT => parse_class_declaration(registry, &compound_id, &name, Class::new_struct()),
    INTERFACE => parse_class_declaration(registry, &compound_id, &name, Class::new_interface()),
    k => println!("Ignoring {:?} in compound declaration", k),
  }

  registry.add_compound(compound_id.to_owned(), kind, name);

  for member in element.children().filter(|e| e.is("member", AnyNS)) {
    parse_member_declaration(registry, member, &compound_id);
  }
}

fn parse_index_file(input_dir: &PathBuf) -> Registry
{
  let mut registry = Registry::new();

  let index_file = input_dir.join("index.xml");
  let root_element = parse_xml_file(&index_file);

  for decl in root_element.children().filter(|e| e.is("compound", AnyNS)) {
    parse_compound_declaration(&mut registry, decl);
  }

  return registry;
}

pub fn parse_xml(input_dir: &PathBuf) -> Registry
{
  let start_time = SystemTime::now();
  println!("Parsing XML input...");

  let mut registry = parse_index_file(input_dir);

  for e in fs::read_dir(input_dir).unwrap() {
    match e {
      Ok(entry) => parse_generic_file(&entry.path(), &mut registry),
      Err(err) => println!("Error encountered when iterating input directory: {}", err),
    }
  }

  let end_time = SystemTime::now();
  println!("Parsed XML files in {} ms",
           end_time.duration_since(start_time).unwrap().as_millis());

  return registry;
}

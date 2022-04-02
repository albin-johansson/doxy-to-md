use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::d2m::doxygen::*;
use crate::d2m::doxygen::CompoundKind::*;

fn generate_group_filename(name: &str) -> String
{
  return format!("group_{}.md", name.to_lowercase().replace(" ", "_"));
}

fn get_class_filename(name: &str) -> String
{
  return format!("class_{}.md",
                 name.to_lowercase()
                     .replace("::", "_")
                     .replace("<", "_")
                     .replace(">", "_")
                     .replace(" ", ""));
}

fn generate_function_comment(writer: &mut BufWriter<&File>, func: &Function) -> io::Result<()>
{
  if !func.docs.brief.is_empty() {
    for docs in &func.docs.brief {
      write!(writer, "\n**Brief:** {}\n", docs)?;
    }
  }

  if !func.docs.pre_conditions.is_empty() {
    write!(writer, "\n**Pre-conditions**\n\n")?;

    for pre in &func.docs.pre_conditions {
      write!(writer, "- {}\n", pre)?;
    }

    write!(writer, "\n")?;
  }

  if !func.docs.post_conditions.is_empty() {
    write!(writer, "\n**Post-conditions**\n\n")?;

    for post in &func.docs.post_conditions {
      write!(writer, "- {}\n", post)?;
    }

    write!(writer, "\n")?;
  }

  if !func.docs.details.is_empty() {
    for details in &func.docs.details {
      write!(writer, "\n{}\n", details)?;
    }
  }

  if !func.docs.notes.is_empty() {
    for note in &func.docs.notes {
      write!(writer, "\n**Note:** {}\n", note)?;
    }
  }

  if !func.docs.warnings.is_empty() {
    for warning in &func.docs.warnings {
      write!(writer, "\n**Warning:** {}\n", warning)?;
    }
  }

  if func.is_member {
    write!(writer, "\n*This is a {} function.*\n", func.access)?;
  }

  generate_parameter_list(writer, &func.parameter_names, &func.docs)?;
  generate_template_parameter_docs(writer, &func.docs.template_parameters)?;

  if !func.docs.exceptions.is_empty() {
    write!(writer, "\n**Exceptions**\n\n")?;

    for (name, desc) in &func.docs.exceptions {
      write!(writer, "- `{}` {}\n", name, desc)?;
    }

    write!(writer, "\n")?;
  }

  if !func.docs.returns.is_empty() {
    write!(writer, "\n**Returns:** {}\n", &func.docs.returns)?;
  }

  Ok(())
}

fn generate_index_file(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
  let path = output_dir.join("index.md");
  println!("Generating file {}", path.display());

  let file = File::create(path)?;
  let mut writer = BufWriter::new(&file);

  write!(writer, "# API\n")?;

  write!(writer, "\n## Modules\n\n")?;
  write!(writer, "\nHere is a list of all modules.\n\n")?;

  for (_, compound) in &registry.compounds {
    // TODO arrange by group relations (subgroups)
    if compound.kind == GROUP {
      write!(writer, "* [{}](groups/{})\n", &compound.title, generate_group_filename(&compound.name))?;
    }
  }

  write!(writer, "\n## Classes\n\n")?;
  write!(writer, "\nHere is a list of all classes.\n\n")?;

  for (compound_id, compound) in &registry.compounds {
    if compound.kind == CLASS || compound.kind == STRUCT || compound.kind == INTERFACE {
      let clazz = registry.classes.get(compound_id).unwrap();
      write!(writer, "* [{}](classes/{})\n",
             &clazz.unqualified_name,
             get_class_filename(&compound.name))?;
    }
  }

  writer.flush()?;
  Ok(())
}

fn generate_template_parameter_docs(writer: &mut BufWriter<&File>,
                                    parameters: &HashMap<String, String>)
  -> io::Result<()>
{
  if !parameters.is_empty() {
    write!(writer, "\n**Template Parameters**\n\n")?;

    for (name, info) in parameters.iter() {
      write!(writer, "* `{}` {}\n", name, if info.is_empty() { "N/A" } else { info })?;
    }

    write!(writer, "\n")?;
  }

  Ok(())
}

fn generate_parameter_list(writer: &mut BufWriter<&File>,
                           parameters: &Vec<String>,
                           docs: &Comment)
  -> io::Result<()>
{
  if !parameters.is_empty() {
    write!(writer, "\n**Parameters**\n\n")?;

    for name in parameters {
      write!(writer, "- `{}`", name)?;

      match docs.parameters.get(name) {
        Some(desc) => write!(writer, " {}", desc)?,
        None => write!(writer, " N/A")?
      }

      write!(writer, "\n")?;
    }
  }

  Ok(())
}

fn generate_function_signature(writer: &mut BufWriter<&File>, func: &Function)
  -> io::Result<()>
{
  if !func.template_args.is_empty() {
    write!(writer, "template <")?;
    let mut first = true;
    for arg in &func.template_args {
      write!(writer, "{}{}", if !first { ", " } else { "" }, arg)?;
      first = false;
    }
    write!(writer, ">\n")?;
  }

  write!(writer, "{}{}{}{}{}{};\n",
         if func.is_static { "static " } else { "" },
         if func.is_explicit { "explicit " } else { "" },
         &func.return_type,
         if func.return_type.is_empty() { "" } else { " " },
         &func.name,
         &func.args)?;

  Ok(())
}

fn generate_function_definition(writer: &mut BufWriter<&File>, func: &Function)
  -> io::Result<()>
{
  write!(writer, "\n### **{}**\n", &func.qualified_name)?;

  write!(writer, "\n```C++\n")?;
  generate_function_signature(writer, func)?;
  write!(writer, "```\n")?;

  generate_function_comment(writer, &func)?;

  if !func.docs.see_also.is_empty() {
    write!(writer, "\n**See Also**\n\n")?;
    for see in &func.docs.see_also {
      write!(writer, "* {}\n", see)?;
    }
  }

  Ok(())
}

fn generate_class_file(destination: &PathBuf,
                       registry: &Registry,
                       compound_id: &RefID,
                       compound: &Compound) -> io::Result<()>
{
  // println!("Generating file {}", destination.display());

  let class = registry.classes.get(compound_id).unwrap();

  let file = File::create(destination)?;
  let mut writer = BufWriter::new(&file);
  write!(writer, "# {}\n", &compound.name)?;

  for par in &compound.docs.brief {
    write!(writer, "\n{}\n", par)?;
  }

  if !compound.docs.details.is_empty() {
    write!(writer, "\n[More...](#detailed-description)\n")?;
  }

  write!(writer, "\n```C++\n")?;
  if !class.template_args.is_empty() {
    write!(writer, "template <")?;
    for arg in &class.template_args {
      write!(writer, "{}", arg)?;
    }
    write!(writer, ">\n")?;
  }
  write!(writer,
         "{} {};\n",
         if class.is_struct { "struct" } else { "class" },
         &class.unqualified_name)?;
  write!(writer, "```\n")?;

  // TODO typedefs

  if !compound.functions.is_empty() {
    write!(writer, "\n## API\n")?;

    let count = compound.functions.len();
    let mut index: usize = 0;

    write!(writer, "\n```C++\n")?;
    for func_id in &compound.functions {
      let func = registry.functions.get(func_id).unwrap();
      generate_function_signature(&mut writer, func)?;

      index += 1;
      if index != count {
        write!(writer, "\n")?;
      }
    }
    write!(writer, "```\n")?;
  }

  if !compound.docs.details.is_empty() {
    write!(writer, "\n## Detailed Description\n\n")?;
    for par in &compound.docs.details {
      write!(writer, "\n{}\n", par)?;
    }
  }

  for note in &compound.docs.notes {
    write!(writer, "\n**Note**: {}\n", note)?;
  }

  for see in &compound.docs.see_also {
    write!(writer, "\n**See**: {}\n", see)?;
  }

  write!(writer, "\n## Members\n")?;

  for func_id in &compound.functions {
    let func = registry.functions.get(func_id).unwrap();
    generate_function_definition(&mut writer, &func)?;
  }

  writer.flush()?;
  Ok(())
}

fn generate_enum_definition(writer: &mut BufWriter<&File>,
                            enumeration: &Enum)
  -> io::Result<()>
{
  write!(writer, "\n## {}\n", &enumeration.qualified_name)?;

  write!(writer, "\n```C++\n")?;
  write!(writer, "enum{} {} \n{{\n",
         if enumeration.is_scoped { " class" } else { "" },
         &enumeration.name)?;

  for value in &enumeration.values {
    write!(writer, "  {}", &value.name)?;

    if !value.initializer.is_empty() {
      write!(writer, " = {}", &value.initializer)?;
    }

    write!(writer, ",\n")?;
  }

  write!(writer, "}};\n")?;
  write!(writer, "```\n")?;

  // write!(writer, "\n| Enumerator | Description |\n")?;
  // write!(writer, "|-----------:|:------------|\n")?;
  // for value in &enumeration.values {
  //   if !value.docs.brief.is_empty() {
  //     write!(writer, "|`{}`|{}|\n", &value.name, &value.docs.brief.join(" "))?;
  //   }
  // }

  Ok(())
}

fn generate_group_file(destination: &PathBuf,
                       registry: &Registry,
                       compound: &Compound) -> io::Result<()>
{
  println!("Generating file {}", destination.display());

  let file = File::create(destination)?;
  let mut writer = BufWriter::new(&file);

  write!(writer, "# {}\n", &compound.title)?;

  for par in &compound.docs.brief {
    write!(writer, "\n{}\n", &par)?;
  }

  for par in &compound.docs.details {
    write!(writer, "\n{}\n", &par)?;
  }

  if !compound.groups.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Groups\n\n")?;

    for group_id in &compound.groups {
      let group = registry.compounds.get(group_id).unwrap();
      write!(writer, "- {}\n", &group.title)?;
    }
  }

  if !compound.classes.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Classes\n\n")?;

    for class_id in &compound.classes {
      let class = registry.classes.get(class_id).unwrap();
      let class_compound = registry.compounds.get(class_id).unwrap();
      let filename = get_class_filename(&class_compound.name);
      write!(writer,
             "- [{} {}](../classes/{})\n",
             if class.is_struct { "struct" } else { "class" },
             &class.unqualified_name,
             &filename)?;
    }
  }

  if !compound.enums.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Enums\n")?;
    write!(writer, "\nThese are the enums associated with this group.\n")?;

    for enum_id in &compound.enums {
      let enumeration = registry.enums.get(enum_id).unwrap();
      generate_enum_definition(&mut writer, enumeration)?;
    }
  }

  if !compound.functions.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Functions\n")?;
    write!(writer, "\nThese are the free functions associated with this group.\n")?;
    for func_id in &compound.functions {
      let func = registry.functions.get(func_id).unwrap();
      if !func.is_member {
        generate_function_definition(&mut writer, func)?;
      }
    }
  }

  if !compound.variables.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Variables\n")?;
    write!(writer, "\nThese are the variables associated with this group.\n")?;

    for variable_id in &compound.variables {
      let variable = registry.variables.get(variable_id).unwrap();

      write!(writer, "\n### {}\n", &variable.qualified_name)?;

      if !variable.docs.brief.is_empty() {
        for brief in &variable.docs.brief {
          write!(writer, "\n{}\n", brief)?;
        }
      }

      write!(writer, "```C++\n")?;
      write!(writer, "{};\n", &variable.definition)?;
      write!(writer, "```\n")?;
    }
  }

  if !compound.defines.is_empty() {
    write!(writer, "\n---")?;
    write!(writer, "\n## Macros\n\n")?;

    for define_id in &compound.defines {
      let define = registry.defines.get(define_id).unwrap();
      write!(writer, "* {}\n", &define.name)?;
    }
  }

  writer.flush()?;
  Ok(())
}

pub fn generate_markdown(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
  let start_time = SystemTime::now();
  println!("Generating Markdown output...");

  generate_index_file(output_dir, registry)?;

  let group_dir = output_dir.join("groups");
  let class_dir = output_dir.join("classes");

  for (compound_id, compound) in &registry.compounds {
    if compound.kind == GROUP {
      let dst = group_dir.join(generate_group_filename(&compound.name));
      generate_group_file(&dst, registry, compound)?;
    } else if compound.kind == CLASS || compound.kind == STRUCT {
      let dst = class_dir.join(get_class_filename(&compound.name));
      generate_class_file(&dst, registry, compound_id, compound)?;
    }
  }

  let end_time = SystemTime::now();
  println!("Generated Markdown files in {} ms",
           end_time.duration_since(start_time).unwrap().as_millis());

  Ok(())
}

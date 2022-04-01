use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use crate::d2m::doxygen::{Compound, Function, RefID, Registry};
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

fn generate_index_file(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
  let path = output_dir.join("index.md");
  println!("Generating file {}", path.display());

  let file = File::create(path)?;
  let mut writer = BufWriter::new(&file);

  write!(writer, "# API\n")?;
  write!(writer, "\nHere is a list of all modules.\n")?;

  write!(writer, "\n## Modules\n\n")?;

  for (_, compound) in &registry.compounds {
    // TODO arrange by group relations (subgroups)
    if compound.kind == GROUP {
      write!(writer, "* [{}](groups/{})\n", &compound.title, generate_group_filename(&compound.name))?;
    }
  }

  Ok(())
}

fn generate_template_parameter_docs(writer: &mut BufWriter<&File>, parameters: &HashMap<String, String>)
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

fn generate_function_definition(writer: &mut BufWriter<&File>, func: &Function) -> io::Result<()>
{
  write!(writer, "\n---\n")?;
  write!(writer, "\n### **{}**\n", &func.qualified_name)?;

  if !func.docs.brief.is_empty() {
    for docs in &func.docs.brief {
      write!(writer, "\n**Synopsis**: {}\n", docs)?;
    }
  } else {
    write!(writer, "\nThis function has no brief documentation.\n")?;
  }

  if !func.docs.details.is_empty() {
    for docs in &func.docs.details {
      write!(writer, "\n{}\n", docs)?;
    }
  }

  if func.is_member {
    write!(writer, "\n*This is a {} function.*\n", func.access)?;
  }

  if !func.docs.notes.is_empty() {
    for note in &func.docs.notes {
      write!(writer, "\n!!! note\n")?;
      write!(writer, "    {}\n", note)?;
    }
  }

  if !func.docs.warnings.is_empty() {
    for warning in &func.docs.warnings {
      write!(writer, "\n!!! warning\n")?;
      write!(writer, "    {}\n", warning)?;
    }
  }

  if !func.docs.pre_conditions.is_empty() {
    write!(writer, "\n**Pre-conditions**\n\n")?;

    for cond in &func.docs.pre_conditions {
      write!(writer, "* {}\n", cond)?;
    }

    write!(writer, "\n")?;
  }

  if !func.docs.post_conditions.is_empty() {
    write!(writer, "\n**Post-conditions**\n\n")?;

    for cond in &func.docs.post_conditions {
      write!(writer, "* {}\n", cond)?;
    }

    write!(writer, "\n")?;
  }

  generate_template_parameter_docs(writer, &func.docs.template_parameters)?;

  if !func.parameter_names.is_empty() {
    write!(writer, "\n**Parameters**\n\n")?;

    for name in &func.parameter_names {
      write!(writer, "* `{}`", name)?;

      match func.docs.parameters.get(name) {
        Some(desc) => write!(writer, " {}", desc)?,
        None => write!(writer, " N/A")?
      }

      write!(writer, "\n")?;
    }
  }

  if !func.docs.exceptions.is_empty() {
    write!(writer, "\n**Exceptions**\n\n")?;

    for (name, desc) in &func.docs.exceptions {
      write!(writer, "* `{}` {}\n", name, desc)?;
    }

    write!(writer, "\n")?;
  }

  if !func.docs.returns.is_empty() {
    write!(writer, "\n**Returns**\n\n")?;
    write!(writer, "{}\n", &func.docs.returns)?;
  }

  write!(writer, "\n```C++\n")?;
  if !func.template_args.is_empty() {
    write!(writer, "template <")?;
    let mut first = true;
    for arg in &func.template_args {
      write!(writer, "{}{}", if !first { ", " } else { "" }, arg)?;
      first = false;
    }
    write!(writer, ">\n")?;
  }
  write!(writer, "{}{}{} {}{};\n",
         if func.is_static { "static " } else { "" },
         if func.is_explicit { "explicit " } else { "" },
         &func.return_type,
         &func.name,
         &func.args)?;
  write!(writer, "```\n")?;

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

  for par in &compound.brief_docs {
    write!(writer, "\n{}\n", par)?;
  }

  write!(writer, "\n## Detailed Description\n\n")?;

  for par in &compound.detailed_docs {
    write!(writer, "\n{}\n", par)?;
  }

  write!(writer, "\n## Members\n")?;

  for func_id in &compound.functions {
    let func = registry.functions.get(func_id).unwrap();
    generate_function_definition(&mut writer, &func)?;
  }

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

  for par in &compound.brief_docs {
    write!(writer, "\n{}\n", &par)?;
  }

  for par in &compound.detailed_docs {
    write!(writer, "\n{}\n", &par)?;
  }

  write!(writer, "\n## Groups\n\n")?;
  if compound.groups.is_empty() {
    write!(writer, "There are no associated subgroups for this group.\n")?;
  } else {
    for group_id in &compound.groups {
      let group = registry.compounds.get(group_id).unwrap();
      write!(writer, "* {}\n", &group.title)?;
    }
  }

  write!(writer, "\n## Classes & Structs\n\n")?;
  if compound.classes.is_empty() {
    write!(writer, "There are no classes or structs associated with this group.\n")?;
  } else {
    for class_id in &compound.classes {
      let class = registry.classes.get(class_id).unwrap();
      let class_compound = registry.compounds.get(class_id).unwrap();
      let filename = get_class_filename(&class_compound.name);
      write!(writer,
             "* [{} {}](../classes/{})\n",
             if class.is_struct { "struct" } else { "class" },
             &class.unqualified_name,
             &filename)?;
    }
  }

  write!(writer, "\n## Functions\n\n")?;
  if compound.functions.is_empty() {
    write!(writer, "There are no functions associated with this group.\n")?;
  } else {
    write!(writer, "These are the free functions associated with this group.\n")?;
    for func_id in &compound.functions {
      let func = registry.functions.get(func_id).unwrap();
      if !func.is_member {
        generate_function_definition(&mut writer, func)?;
      }
    }
  }

  write!(writer, "\n## Variables\n\n")?;
  if compound.variables.is_empty() {
    write!(writer, "There are no variables associated with this group.\n")?;
  } else {
    write!(writer, "These are the variables associated with this group.\n")?;

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

      write!(writer, "\n---\n")?;
    }
  }

  write!(writer, "\n## Defines\n\n")?;
  if compound.defines.is_empty() {
    write!(writer, "There are no defines associated with this group.\n")?;
  } else {
    for define_id in &compound.defines {
      let define = registry.defines.get(define_id).unwrap();
      write!(writer, "* {}\n", &define.name)?;
    }
  }

  Ok(())
}

pub fn generate_markdown(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
  let start_time = std::time::SystemTime::now();
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

  let end_time = std::time::SystemTime::now();
  println!("Generated Markdown files in {} ms",
           end_time.duration_since(start_time).unwrap().as_millis());

  Ok(())
}

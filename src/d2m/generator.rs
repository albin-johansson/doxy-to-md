use crate::d2m::doxygen::{Compound, Registry, RefID, Function};
use crate::d2m::doxygen::CompoundKind::*;

use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::io::Write;

fn generate_group_filename(name: &str) -> String
{
    return format!("group_{}.md", name.to_lowercase().replace(" ", "_"));
}

fn get_class_filename(name: &str) -> String
{
    return format!("class_{}.md", name.to_lowercase()
        .replace("::", "_")
        .replace("<", "_")
        .replace(">", "_")
        .replace(" ", ""));
}

fn generate_index_file(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
    let path = output_dir.join("index.md");
    println!("Generating file {}", path.display());

    let mut file = File::create(path).unwrap();

    write!(file, "# API\n")?;
    write!(file, "\nHere is a list of all modules.\n")?;

    write!(file, "\n## Modules\n\n")?;

    for (compound_id, compound) in &registry.compounds {
        // TODO arrange by group relations (subgroups)
        if compound.kind == GROUP {
            write!(file, "* [{}](groups/{})\n", &compound.title, generate_group_filename(&compound.name))?;
        }
    }

    return Ok(());
}

fn generate_function_definition(file: &mut File, func: &Function) -> io::Result<()>
{
    write!(file, "\n### **{}**\n", &func.name)?;

    // TODO brief, details, template parameters, parameter docs, exceptions

    if func.is_member {
        write!(file, "\n*This is a {} function.*\n", func.access)?;
    }

    write!(file, "\n```C++\n")?;
    write!(file, "{} {}{};\n", &func.return_type, &func.name, &func.arguments)?;
    write!(file, "```\n")?;

    return Ok(());
}

fn generate_class_file(destination: &PathBuf,
                       registry: &Registry,
                       compound_id: &RefID,
                       compound: &Compound) -> io::Result<()>
{
    println!("Generating file {}", destination.display());

    let class = registry.classes.get(compound_id).unwrap();

    let file = File::create(destination);
    if let Ok(mut file) = file {
        write!(file, "# {}\n", &compound.name)?;

        write!(file, "\n```C++\n")?;
        write!(file, "{} {};\n",
               if class.is_struct { "struct" } else { "class" },
               &class.unqualified_name)?;
        write!(file, "```\n")?;

        // TODO brief

        write!(file, "\n## Detailed Description\n\n")?;

        // TODO details

        write!(file, "\n## Member Functions\n")?;

        for func_id in &compound.functions {
            let func = registry.functions.get(func_id).unwrap();
            generate_function_definition(&mut file, &func)?;
        }
    }

    return Ok(());
}

fn generate_group_file(destination: &PathBuf,
                       registry: &Registry,
                       compound_id: &RefID,
                       compound: &Compound) -> io::Result<()>
{
    println!("Generating file {}", destination.display());
    let file = File::create(destination);
    if let Ok(mut file) = file {
        write!(file, "# {}\n", &compound.title)?;

        write!(file, "\n## Groups\n\n")?;
        if compound.groups.is_empty() {
            write!(file, "There are no groups owned by this group.\n")?;
        } else {
            for group_id in &compound.groups {
                let group = registry.compounds.get(group_id).unwrap();
                write!(file, "* {}\n", &group.title)?;
            }
        }

        write!(file, "\n## Classes & Structs\n\n")?;
        if compound.classes.is_empty() {
            write!(file, "There are no classes or structs associated with this group.\n")?;
        } else {
            for class_id in &compound.classes {
                let c = registry.classes.get(class_id).unwrap();
                let filename = get_class_filename(&compound.name);
                write!(file, "* [{} {}](../classes/{})\n",
                       if c.is_struct { "struct" } else { "class" },
                       &c.unqualified_name,
                       &filename)?;
            }
        }

        write!(file, "\n## Functions\n\n")?;
        if compound.functions.is_empty() {
            write!(file, "There are no functions associated with this group.\n")?;
        } else {
            write!(file, "These are the free functions associated with this group.\n")?;
            for func_id in &compound.functions {
                let func = registry.functions.get(func_id).unwrap();
                if !func.is_member {
                    generate_function_definition(&mut file, func)?;
                }
            }
        }

        write!(file, "\n## Variables\n\n")?;
        if compound.variables.is_empty() {
            write!(file, "There are no variables associated with this group.\n")?;
        } else {
            for variable_id in &compound.variables {
                let variable = registry.variables.get(variable_id).unwrap();
                write!(file, "* {}\n", &variable.name)?;
            }
        }

        write!(file, "\n## Defines\n\n")?;
        if compound.defines.is_empty() {
            write!(file, "There are no defines associated with this group.\n")?;
        } else {
            for define_id in &compound.defines {
                let define = registry.defines.get(define_id).unwrap();
                write!(file, "* {}\n", &define.name)?;
            }
        }
    }

    return Ok(());
}

pub fn generate_markdown(output_dir: &PathBuf, registry: &Registry) -> io::Result<()>
{
    println!("Generating Markdown output...");

    generate_index_file(output_dir, registry)?;

    let group_dir = output_dir.join("groups");
    let class_dir = output_dir.join("classes");

    for (compound_id, compound) in &registry.compounds {
        if compound.kind == GROUP {
            let dst = group_dir.join(generate_group_filename(&compound.name));
            generate_group_file(&dst, registry, compound_id, compound)?;
        } else if compound.kind == CLASS || compound.kind == STRUCT {
            let dst = class_dir.join(get_class_filename(&compound.name));
            generate_class_file(&dst, registry, compound_id, compound)?;
        }
    }

    return Ok(());
}
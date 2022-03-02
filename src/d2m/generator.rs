use crate::d2m::doxygen;
use crate::d2m::doxygen::CompoundKind::*;
use crate::d2m::doxygen::Registry;

use std::path;
use std::fs;
use std::path::PathBuf;
use std::io::Write;

fn generate_index_file(output_dir: &path::PathBuf, registry: &Registry) {
    let path = output_dir.join("index.md");
    let mut file = fs::File::create(path).unwrap();

    write!(file, "# API\n").unwrap();
    write!(file, "\nHere is a list of all modules.\n").unwrap();

    write!(file, "\n## Modules\n\n").unwrap();

    for (compound_id, compound) in registry.compounds() {
        if compound.kind == GROUP {
            write!(file, "* {}\n", compound.title).unwrap();
        }
    }
}

pub fn generate_markdown(output_dir: &path::PathBuf, registry: &Registry) {
    println!("Generating Markdown output...");

    generate_index_file(output_dir, registry);
}
mod d2m;

use std::fs;
use std::io;
use std::path::Path;

use clap::Parser;
use path_absolutize::*;

use crate::d2m::generator;
use crate::d2m::parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(short, long)]
  input_dir: String,

  #[clap(short, long)]
  output_dir: String,
}

fn main() -> io::Result<()> {
  let args = Args::parse();
  let input_dir = Path::new(&args.input_dir).absolutize()?.to_path_buf();
  let output_dir = Path::new(&args.output_dir).absolutize()?.to_path_buf();

  assert!(input_dir.is_absolute());
  assert!(output_dir.is_absolute());

  println!("Input directory: {}", input_dir.display());
  println!("Output directory: {}", output_dir.display());

  if !input_dir.exists() {
    panic!("Input directory does not exist!");
  }

  // Makes sure that the directories we'll write to exist
  fs::create_dir_all(&output_dir).unwrap();
  fs::create_dir_all(output_dir.join("groups"))?;
  fs::create_dir_all(output_dir.join("classes"))?;

  let registry = parser::parse_xml(&input_dir);
  generator::generate_markdown(&output_dir, &registry)
}

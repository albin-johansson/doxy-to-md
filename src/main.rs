mod d2m;

use crate::d2m::generator;
use crate::d2m::parser;

use clap::Parser;
use std::fs;
use std::io;
use std::path::PathBuf;

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
  let input_dir = PathBuf::from(&args.input_dir);
  let output_dir = PathBuf::from(&args.output_dir);

  println!("--input-dir: {}", input_dir.display());
  println!("--output-dir: {}", output_dir.display());

  if !input_dir.is_absolute() {
    panic!("Input directory must be absolute!");
  }

  if !output_dir.is_absolute() {
    panic!("Output directory must be absolute!");
  }

  if !input_dir.exists() {
    panic!("Input directory does not exist!");
  }

  fs::create_dir_all(&output_dir).unwrap();
  fs::create_dir_all(output_dir.join("groups"))?;
  fs::create_dir_all(output_dir.join("classes"))?;

  let registry = parser::parse_xml(&input_dir);
  return generator::generate_markdown(&output_dir, &registry);
}

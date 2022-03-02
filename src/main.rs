use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    input_dir: String,

    #[clap(short, long)]
    output_dir: String,
}

fn main() {
    let args = Args::parse();

    let input_dir = PathBuf::from(&args.input_dir);
    let output_dir = PathBuf::from(&args.output_dir);

    if !input_dir.exists() {
        panic!("Input directory does not exist!");
    }

    if !output_dir.exists() {
        panic!("Output directory does not exist!");
    }

    let canonical_input_dir = input_dir.canonicalize().unwrap();
    let canonical_output_dir = output_dir.canonicalize().unwrap();

    println!("Input directory: {}", canonical_input_dir.display());
    println!("Output directory: {}", canonical_output_dir.display());

    fs::create_dir_all(&canonical_input_dir).unwrap();
}
